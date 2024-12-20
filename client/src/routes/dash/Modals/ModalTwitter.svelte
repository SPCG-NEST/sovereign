<script lang="ts">
	import * as Dialog from '$lib/components/ui/dialog';

	let { children } = $props();
</script>

<Dialog.Root>
	<Dialog.Trigger class="w-full">
		{@render children?.()}
	</Dialog.Trigger>
	<Dialog.Content class="max-h-[90vh] overflow-y-auto">
		<Dialog.Header>
			<Dialog.Title>Confirm your X Account</Dialog.Title>
			<Dialog.Description>
				<div>
					<button
						class="mt-4 w-full rounded-xl border-2 border-black bg-black p-2 transition-all hover:bg-black"
					>
						Purchase
					</button>
				</div>
			</Dialog.Description>
		</Dialog.Header>
	</Dialog.Content>
</Dialog.Root>
