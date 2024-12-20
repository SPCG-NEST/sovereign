import { SvelteKitAuth } from '@auth/sveltekit';
import XProvider from '@auth/sveltekit/providers/twitter'; // Hypothetical provider

export const { handle, signIn, signOut } = SvelteKitAuth({
	providers: [
		XProvider({
			clientId: process.env.X_CLIENT_ID,
			clientSecret: process.env.X_CLIENT_SECRET,
			redirectProxyUrl: process.env.X_CALLBACK_URL
		})
	]
	// secret: process.env.AUTH_SECRET
});
