// See https://svelte.dev/docs/kit/types#app.d.ts
// for information about these interfaces
declare global {
	namespace App {
		// interface Error {}
		// interface Locals {}
		// interface PageData {}
		// interface PageState {}
		interface Platform {
			env: {
				X_CLIENT_ID: string;
				X_CLIENT_SECRET: string;
				X_CALLBACK_URL: string;
			};
		}
	}
}

export {};
