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
	}

	if (requestedPath === '/auth/twitter/callback') {
	}

	const response = await resolve(event);
	return response;
};
