<script lang="ts">
	import { page } from '$app/stores';
	import Privy, {
		getAllUserEmbeddedSolanaWallets,
		getUserEmbeddedEthereumWallet,
		getUserEmbeddedSolanaWallet,
		LocalStorage,
		type PrivyEmbeddedSolanaWalletProvider
	} from '@privy-io/js-sdk-core';
	import type { PrivyAuthenticatedUser } from '@privy-io/public-api';

	import {
		ComputeBudgetProgram,
		Connection,
		PublicKey,
		SendTransactionError,
		SystemProgram,
		Transaction,
		TransactionInstruction,
		TransactionMessage,
		VersionedTransaction
	} from '@solana/web3.js';
	import { onDestroy, onMount } from 'svelte';
	import bs58 from 'bs58';
	import type { Signer, TransactionError } from '@solana/web3.js';

	let privy_oauth_state = $page.url.searchParams.get('privy_oauth_state');
	let privy_oauth_code = $page.url.searchParams.get('privy_oauth_code');

	let privy: Privy | null = $state(null);
	let twitURL = $state('');
	let user = $state(null as PrivyAuthenticatedUser | null);
	let iframeSrc = $state('');
	let iframe = $state(null as HTMLIFrameElement | null);
	let handler: (e: MessageEvent) => void;
	let embeddedWallet = $state(null as PrivyEmbeddedSolanaWalletProvider | null);
	let address = $state('');
	let confirmedTx = $state('');
	onMount(async () => {
		const newPrivy = new Privy({
			appId: 'cm4rluhru04zmuj8pzs0hklmk',
			storage: new LocalStorage()
		});
		privy = newPrivy;
		iframeSrc = privy?.embeddedWallet.getURL()! as string;
		console.log('iframeSrc', iframeSrc);
		if (iframe?.contentWindow) {
			privy!.setMessagePoster(iframe.contentWindow as any);
			handler = (e: MessageEvent) => privy!.embeddedWallet.onMessage(e.data);
			window.addEventListener('message', handler);
		}
	});

	onDestroy(() => {
		if (typeof window !== 'undefined' && handler) {
			window.removeEventListener('message', handler);
		}
	});

	async function generateURL() {
		twitURL = (await privy?.auth.oauth.generateURL('twitter', 'http://localhost:5173/privy'))!
			.url as string;
	}

	async function loginWithCode() {
		privy?.auth.oauth
			.loginWithCode(privy_oauth_code as string, privy_oauth_state as string, 'twitter')
			.then((e) => {
				user = e;
				console.log('USER', user);
			});
	}

	async function createEmbeddedWallet() {
		if (privy?.embeddedWallet.hasEmbeddedWallet()) {
			const accounts = getAllUserEmbeddedSolanaWallets(user!.user);
			address = accounts[0].address;

			const provider = await privy.embeddedWallet.getSolanaProvider(
				accounts[0],
				accounts[0].address,
				'solana-address-verifier'
			);
			const connection = new Connection('http://127.0.0.1:8899', 'confirmed');
			const pkey = new PublicKey(address);
			console.log('pkey: ', pkey.toBase58());

			const simpleTx = new VersionedTransaction(
				new TransactionMessage({
					payerKey: pkey,
					recentBlockhash: (await connection.getLatestBlockhash()).blockhash,
					instructions: [
						SystemProgram.transfer({
							fromPubkey: pkey,
							toPubkey: pkey,
							lamports: 1
						})
					]
				}).compileToLegacyMessage()
			);
			const message = Buffer.from(simpleTx.message.serialize()).toString('base64');
			const simpleSig = (
				await provider.request({
					method: 'signMessage',
					params: {
						message: message
					}
				})
			).signature;
			console.log('signature: ', simpleSig);
			simpleTx.addSignature(pkey, Uint8Array.from(Buffer.from(simpleSig, 'base64')));
			console.log('Signed: ', Buffer.from(simpleTx.serialize()).toString('base64'));
			// sign that message ^^^ and attach the signature
			const condfirmedSentTx = await connection.sendTransaction(simpleTx);
			confirmedTx = condfirmedSentTx;

			return;
		} else {
			embeddedWallet = (await privy?.embeddedWallet.createSolana())!.provider;
		}
	}

	async function testMessage() {}
</script>

<p>
	address:{address}
	confirmedTx: {confirmedTx}
</p>
<hr />
<div>
	<button onclick={generateURL}> generateURL </button> <br />
	<a href={twitURL}>Go To Twitter {twitURL}</a> <br />
	<button onclick={loginWithCode}> loginWithCode {privy_oauth_state} {privy_oauth_code} </button>
	<br />
	<button onclick={createEmbeddedWallet}> createEmbeddedWallet {user?.user.id} </button> <br />
	<iframe
		bind:this={iframe}
		src={iframeSrc}
		style="width: 0; height: 0; border: none;"
		title="Privy embedded wallet"
	>
	</iframe>
</div>
