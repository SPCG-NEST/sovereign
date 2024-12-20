<script lang="ts">
	import { PUBLIC_RPC_URL } from '$env/static/public';
	import { cn } from '$lib/utils.js';
	import Body from './Body.svelte';
	import News from './News.svelte';
	import * as Tabs from '$lib/components/ui/tabs';
	import { getAppKit } from '$lib/wallet/appkit.svelte';
	import { walletStore } from '$lib/stores/wallet.svelte';
	import { Connection, LAMPORTS_PER_SOL, PublicKey } from '@solana/web3.js';
	import { onMount } from 'svelte';
	import type { AppKit } from '@reown/appkit';
	import ModalTwitter from './Modals/ModalTwitter.svelte';

	let appkit: AppKit | null = $state(null);
	let address = $derived.by(() => ($walletStore.connected ? $walletStore.address : ''));

	function login() {
		appkit?.open();
	}
	function openAccount() {
		appkit?.open({ view: 'Account' });
	}

	let tab: 'dash' | 'news' | 'state' = $state('dash');

	onMount(() => {
		appkit = getAppKit(); // initialize appkit

		appkit?.subscribeAccount((e) => {
			walletStore.update((state) => ({
				...state,
				address: e.address ?? null,
				connected: !!e.address
			}));
		});
	});

	const rpc = PUBLIC_RPC_URL as string;
	const connection = new Connection(rpc, 'confirmed');
	let publicKey = $derived(address === '' ? null : new PublicKey(address as string));
	let resolvedBalance = $state(0);

	let balance = $derived.by(() => {
		if (connection && address !== '') {
			connection // connection to solana rpc endpoint
				.getBalance(publicKey as PublicKey) // fetch balance from connection
				.then((data) => {
					resolvedBalance = data / LAMPORTS_PER_SOL; // correct the decimal placement
					walletStore.update((state) => ({
						...state,
						balance: resolvedBalance
					}));
				})
				.catch((error) => {
					console.error('Error fetching balance:', error);
					resolvedBalance = 0; // or whatever error value you want
				});
		}
		return resolvedBalance;
	});
</script>

<div
	class="grid h-screen w-screen md:grid-cols-[2fr_1fr] md:grid-rows-[5rem_1fr] md:overflow-x-hidden"
>
	{#if !!address}
		<div class="bg-orange-400 p-2 text-black md:hidden">
			<ModalTwitter>
				<button>Verify your X Account</button>
			</ModalTwitter>
		</div>
	{/if}
	<div
		class="order-last justify-center border-t-8 border-panel border-b-panel px-4 md:order-first md:flex md:justify-between md:border-b-8 md:border-r-8 md:border-t-0"
	>
		<div class="flex items-center">
			<button
				class={cn(
					'hidden text-4xl font-bold md:inline',
					tab === 'dash' ? 'text-foreground' : 'text-panel'
				)}
				onclick={() => {
					tab = 'dash';
				}}
			>
				SOVEREIGN
			</button>
			<span class="hidden text-4xl font-bold text-panel md:ml-2 md:inline"> | </span>
			<button
				class={cn(
					'hidden text-4xl font-bold md:inline',
					tab === 'state' ? 'text-foreground' : 'text-panel'
				)}
				onclick={() => {
					tab = 'state';
				}}
			>
				NATION STATE
			</button>
		</div>
		<div class="flex items-center pt-4 md:pt-0">
			{#if !!address}
				<button onclick={openAccount} class="rounded-xl bg-panel px-4 py-2">
					<div class="group flex items-center gap-4">
						<span class="tracking-tight">
							{balance} SOL |
							{address.substring(0, 4)}
							{address.substring(address.length - 4, address.length)}
						</span>
						<img
							class="scale-100 rounded-full bg-background bg-center p-[2px] transition-all group-hover:scale-125"
							src={`https://api.dicebear.com/9.x/identicon/svg?seed=${address}`}
							alt="pfp"
							width="35px"
							height="35px"
						/>
					</div>
				</button>
			{:else}
				<button
					onclick={login}
					class="scale-100 rounded-xl bg-panel px-4 py-2 group-hover:scale-110"
				>
					login
				</button>
			{/if}
		</div>
		<Tabs.Root bind:value={tab} class="w-full py-4 md:hidden">
			<Tabs.List class="justify-self-end">
				<Tabs.Trigger value="dash">Dashboard</Tabs.Trigger>
				<Tabs.Trigger value="news">News</Tabs.Trigger>
				<Tabs.Trigger value="state">States</Tabs.Trigger>
			</Tabs.List>
		</Tabs.Root>
	</div>
	<div class="hidden items-center justify-center border-b-8 border-b-panel md:flex">
		<p class="text-4xl font-bold text-panel">NEWS</p>
	</div>

	<!-- BOTTOM SECTION -->
	<div
		class={`overflow-y-auto overflow-x-hidden border-r-panel md:h-[calc(100vh-5rem)] md:overflow-x-visible md:border-r-8`}
	>
		{#if tab === 'dash'}
			<Body></Body>
		{:else if tab === 'news'}
			<div class="flex w-full flex-col gap-2 overflow-y-auto overflow-x-hidden p-4">
				<News></News>
			</div>
		{:else if tab === 'state'}
			<div class="flex w-full flex-col gap-2 overflow-y-auto overflow-x-hidden p-4">
				todo: add state panels
			</div>
		{/if}
	</div>
	<div class="hidden w-full flex-col gap-4 overflow-y-auto overflow-x-hidden p-4 md:flex">
		<News></News>
	</div>
</div>
