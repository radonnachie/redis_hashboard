<script lang="ts">
	export let name: string;
	export let content: Map<string, string>;
</script>

<div>
	<h1>{name}</h1>
	<div>
		{#each [...content] as kv_tuple (kv_tuple[0])}
		<div style="display:flex;">
			<div>{kv_tuple[0]}</div>
			<div>{kv_tuple[1]}</div>
		</div>
		{/each}
	</div>
</div>