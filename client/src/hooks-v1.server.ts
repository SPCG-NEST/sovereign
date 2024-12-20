import { redirect } from '@sveltejs/kit';
import type { Handle } from '@sveltejs/kit';
import * as env from '$env/static/private';
import crypto from 'crypto';
import axios from 'axios';

const CALLBACK_URL = env.X_CALLBACK_URL;

export const handle: Handle = async ({ event, resolve }) => {
	const requestedPath = event.url.pathname;
	const cookies = event.cookies;

	console.log({ SECRET: env.X_CLIENT_SECRET, CLIENTID: env.X_CLIENT_ID });

	if (requestedPath === '/auth/twitter') {
		// Step 1: Generate OAuth parameters
		const oauthNonce = crypto.randomBytes(32).toString('hex');
		const oauthTimestamp = Math.floor(Date.now() / 1000).toString();

		// Create parameter object in alphabetical order (important for signature)
		const parameters = {
			oauth_callback: CALLBACK_URL,
			oauth_consumer_key: env.X_CLIENT_ID,
			oauth_nonce: oauthNonce,
			oauth_signature_method: 'HMAC-SHA1',
			oauth_timestamp: oauthTimestamp,
			oauth_version: '1.0'
		};

		// Create parameter string for signature
		const paramString = Object.keys(parameters)
			.sort()
			.map(
				(key) =>
					`${encodeURIComponent(key)}=${encodeURIComponent(parameters[key as keyof typeof parameters])}`
			)
			.join('&');

		// Generate signature base string
		const signatureBaseString = [
			'POST',
			'https://api.twitter.com/oauth/request_token',
			paramString
		]
			.map(encodeURIComponent)
			.join('&');

		// Create signing key
		const signingKey = `${encodeURIComponent(env.X_CLIENT_SECRET)}&`;

		// Generate signature
		const oauthSignature = crypto
			.createHmac('sha1', signingKey)
			.update(signatureBaseString)
			.digest('base64');

		// Create Authorization header
		const authHeader =
			'OAuth ' +
			Object.entries({
				...parameters,
				oauth_signature: oauthSignature
			})
				.map(
					([key, value]) =>
						`${encodeURIComponent(key)}="${encodeURIComponent(value)}"`
				)
				.join(', ');

		try {
			const response = await axios.post(
				'https://api.twitter.com/oauth/request_token',
				null,
				{
					headers: {
						Authorization: authHeader,
						'Content-Type': 'application/x-www-form-urlencoded'
					}
				}
			);

			const responseData = new URLSearchParams(response.data);
			const oauthToken = responseData.get('oauth_token');

			if (!oauthToken) {
				throw new Error('Failed to obtain request token from Twitter');
			}

			// Step 2: Redirect to Twitter's authentication page
			throw redirect(
				302,
				`https://api.twitter.com/oauth/authenticate?oauth_token=${oauthToken}`
			);
		} catch (error) {
			console.error('Error during Twitter login:', error);
			if (axios.isAxiosError(error)) {
				console.error('Response data:', error.response?.data);
				console.error('Response status:', error.response?.status);
			}
			return new Response('Failed to start Twitter login', { status: 500 });
		}
	}

	// If user is returning from Twitter (callback)
	if (requestedPath === '/auth/callback') {
		const oauthToken = event.url.searchParams.get('oauth_token');
		const oauthVerifier = event.url.searchParams.get('oauth_verifier');

		if (!oauthToken || !oauthVerifier) {
			return new Response('Missing OAuth token or verifier', { status: 400 });
		}

		const oauthNonce = crypto.randomBytes(32).toString('hex');
		const oauthTimestamp = Math.floor(Date.now() / 1000).toString();

		const params = new URLSearchParams({
			oauth_consumer_key: env.X_CLIENT_ID,
			oauth_nonce: oauthNonce,
			oauth_signature_method: 'HMAC-SHA1',
			oauth_timestamp: oauthTimestamp,
			oauth_version: '1.0',
			oauth_token: oauthToken
		});

		const signatureBaseString = `POST&${encodeURIComponent('https://api.twitter.com/oauth/access_token')}&${encodeURIComponent(params.toString())}`;
		const signingKey = `${encodeURIComponent(env.X_CLIENT_SECRET)}&`;
		const oauthSignature = crypto
			.createHmac('sha1', signingKey)
			.update(signatureBaseString)
			.digest('base64');

		params.append('oauth_signature', oauthSignature);

		try {
			const response = await axios.post(
				'https://api.twitter.com/oauth/access_token',
				null,
				{
					headers: {
						Authorization: `OAuth ${params.toString()}`,
						'Content-Type': 'application/x-www-form-urlencoded'
					}
				}
			);

			const responseData = new URLSearchParams(response.data);
			const accessToken = responseData.get('oauth_token');
			const accessTokenSecret = responseData.get('oauth_token_secret');
			const userId = responseData.get('user_id');
			const screenName = responseData.get('screen_name');

			console.log('Twitter user authenticated:', {
				accessToken,
				userId,
				screenName
			});

			// Save the user session data (e.g., store in cookies, local storage, or database)
			event.cookies.set('twitter_access_token', accessToken!, {
				path: '/',
				httpOnly: true
			});
			event.cookies.set('twitter_screen_name', screenName!, {
				path: '/',
				httpOnly: true
			});

			throw redirect(302, '/');
		} catch (error) {
			console.error('Error exchanging request token for access token:', error);
			return new Response('Failed to authenticate with Twitter', {
				status: 500
			});
		}
	}

	const response = await resolve(event);
	return response;
};
