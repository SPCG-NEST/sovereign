import { PUBLIC_RPC_URL } from '$env/static/public';
import { updateWalletStore, walletStore } from '$lib/stores/wallet.svelte';
import Privy, {
	getAllUserEmbeddedSolanaWallets,
	type PrivyEmbeddedSolanaWalletProvider
} from '@privy-io/js-sdk-core';
import type { PrivyAuthenticatedUser } from '@privy-io/public-api';
import { Connection, LAMPORTS_PER_SOL, PublicKey } from '@solana/web3.js';

async function createEmbeddedWallet(props: {
	privy: Privy;
	user: PrivyAuthenticatedUser;
	setAddress?: (newAddress: string | null) => void;
	setProvider?: (newProvider: PrivyEmbeddedSolanaWalletProvider | null) => void;
	setEmbeddedWallet?: (
		newEmbeddedWallet: PrivyEmbeddedSolanaWalletProvider | null
	) => void;
}) {
	const { user, privy } = props;
	if (!user || !privy) return console.error('No user or privy');
	const [account] = getAllUserEmbeddedSolanaWallets(user!.user);
	const hasEmbeddedWallet = await privy?.embeddedWallet.hasEmbeddedWallet();

	if (account && hasEmbeddedWallet) {
		if (props.setAddress) {
			props.setAddress(account.address);
			walletStore.update((state) => ({ ...state, address: account.address }));
		}

		const newProvider = await privy.embeddedWallet.getSolanaProvider(
			account,
			account.address,
			'solana-address-verifier'
		);
		if (props.setProvider) {
			props.setProvider(newProvider);
		}

		await getWalletBalance(account.address);
	} else {
		if (!props.setEmbeddedWallet) return console.error('No embedded wallet');
		const solanaProvider = await privy?.embeddedWallet.createSolana();
		props.setEmbeddedWallet(solanaProvider!.provider);
	}
}

async function getWalletBalance(address: string) {
	const connection = new Connection(PUBLIC_RPC_URL as string);
	updateWalletStore.setConnection(connection);

	if (!connection || !address) {
		return 0;
	}

	try {
		const publicKey = new PublicKey(address);
		const balance = await connection.getBalance(publicKey);
		const resolvedBalance = balance / LAMPORTS_PER_SOL;

		walletStore.update((state) => ({
			...state,
			balance: resolvedBalance,
			address: state.address ?? address
		}));

		return resolvedBalance;
	} catch (error) {
		console.error('Error fetching balance:', error);
		return 0;
	}
}

export const walletHandler = {
	createEmbeddedWallet,
	getWalletBalance
};
