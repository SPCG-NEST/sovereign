<script>
	import Citizens from './Citizens/Citizens.svelte';
	import MintNewCitizen from './Panels/MintNewCitizen.svelte';
</script>

<div class="flex h-full flex-col gap-8">
	<div class="flex flex-col gap-4 p-4">
		<div class="shadow-flat flex flex-col gap-2 rounded-xl border-2 border-black bg-panel p-4">
			<div class="flex flex-col justify-between sm:flex-row">
				<p class="text-light text-4xl">Citizens</p>
				<div class="ml-3 flex gap-[-10px] sm:ml-0">
					{#each Array.from({ length: 8 }).map((_, i) => i) as slotNumber}
						<div
							class="ml-[-1rem] h-10 w-10 overflow-hidden rounded-full border-2 border-panel bg-foreground"
						>
							<img
								src={`https://api.dicebear.com/9.x/lorelei-neutral/svg?seed=${slotNumber}`}
								alt="avatar"
							/>
						</div>
					{/each}
				</div>
			</div>
			<p class="font-light">
				Hire professionals to join your efforts. Each citizen starts with 1 skill. Once hired, you
				can contract out citizens to nations for cash. Or task a citizen on research, for them to
				learn more skills.
			</p>
		</div>

		<div
			class="shadow-flat w-fit min-w-[300px] gap-2 rounded-xl border-2 border-black bg-panel p-4 sm:max-w-[70%]"
		>
			<MintNewCitizen />
		</div>
	</div>

	<div class="sm:h-[[100%] flex-grow border-t-8 border-t-panel bg-black p-8 sm:overflow-y-auto">
		<Citizens />
	</div>
</div>
