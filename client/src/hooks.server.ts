import { redirect } from '@sveltejs/kit';
import type { Handle } from '@sveltejs/kit';
import * as env from '$env/static/private';
import crypto from 'crypto';
import axios from 'axios';

const TWITTER_OAUTH_URL = 'https://twitter.com/i/oauth2/authorize';
const CALLBACK_URL = env.X_CALLBACK_URL;

export const handle: Handle = async ({ event, resolve }) => {
	const requestedPath = event.url.pathname;
	const cookies = event.cookies;

	// Generate a code verifier and code challenge for PKCE (Proof Key for Code Exchange)
	const codeVerifier = crypto.randomBytes(32).toString('base64url');
	const codeChallenge = crypto
		.createHash('sha256')
		.update(codeVerifier)
		.digest('base64');

	// Save the code verifier in cookies for later use
	cookies.set('twitter_code_verifier', codeVerifier, {
		path: '/',
		httpOnly: true,
		secure: true
	});

	if (requestedPath === '/auth/twitter') {
		// Redirect the user to the Twitter authorization page
		const state = crypto.randomBytes(16).toString('hex');
		cookies.set('twitter_auth_state', state, {
			path: '/',
			httpOnly: true,
			secure: true
		});

		const oauth2Parameters = new URLSearchParams({
			response_type: 'code',
			client_id: env.X_CLIENT_ID,
			redirect_uri: CALLBACK_URL,
			scope: 'users.read',
			state,
			code_challenge: codeChallenge,
			code_challenge_method: 'S256'
		});

		throw redirect(302, `${TWITTER_OAUTH_URL}?${oauth2Parameters.toString()}`);
	}

	if (requestedPath === '/auth/twitter/callback') {
		const url = new URL(event.url);
		const code = url.searchParams.get('code');
		const state = url.searchParams.get('state');
		const storedState = cookies.get('twitter_auth_state');
		const codeVerifier = cookies.get('twitter_code_verifier');

		if (!code || !state || state !== storedState) {
			return new Response('Invalid state or missing code', { status: 400 });
		}

		try {
			const tokenResponse = await axios.post(
				'https://api.twitter.com/2/oauth2/token',
				new URLSearchParams({
					grant_type: 'authorization_code',
					code,
					redirect_uri: CALLBACK_URL,
					client_id: env.X_CLIENT_ID,
					code_verifier: codeVerifier as string
				}),
				{
					headers: { 'Content-Type': 'application/x-www-form-urlencoded' }
				}
			);

			const { access_token } = tokenResponse.data;

			// Use the access token to get the user info from Twitter
			const userResponse = await axios.get(
				'https://api.twitter.com/2/users/me',
				{
					headers: { Authorization: `Bearer ${access_token}` }
				}
			);

			const userData = userResponse.data;

			// Store user data (can be saved to a database or used to create a session)
			cookies.set('twitter_user', JSON.stringify(userData), {
				path: '/',
				httpOnly: true,
				secure: true
			});

			// Redirect the user back to the homepage (or another route)
			throw redirect(302, '/');
		} catch (error) {
			console.error('Error during token exchange or user fetch:', error);
			return new Response('Authentication failed', { status: 500 });
		}
	}

	const response = await resolve(event);
	return response;
};
