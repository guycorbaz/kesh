<script lang="ts">
	let { children } = $props();
</script>

<div class="flex min-h-screen flex-col items-center justify-center bg-surface">
	<!-- Logo -->
	<div class="mb-8">
		<a href="/" class="text-3xl font-semibold text-primary">Kesh</a>
	</div>

	<!-- Contenu wizard -->
	<div class="w-full max-w-lg">
		{@render children()}
	</div>

	<!-- Footer disclaimer FR7 -->
	<footer class="mt-8 text-center text-xs text-text-muted">
		Kesh v0.1.0 &mdash; Logiciel libre (EUPL 1.2). Les donn&eacute;es ne remplacent pas un fiduciaire.
	</footer>
</div>
