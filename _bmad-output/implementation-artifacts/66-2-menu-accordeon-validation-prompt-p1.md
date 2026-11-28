# Prompt — validation P1 de la fiche `66-2-menu-accordeon` (#852, #959), passe complète

Tu valides, **en lecture seule**, la fiche `_bmad-output/implementation-artifacts/66-2-menu-accordeon.md` de Kesh (comptabilité suisse ; frontend SvelteKit). Worktree : `/home/gcorbaz/devel/kesh-66-2` (branche `story/66-2-menu-accordeon`). Contexte frais : tu n'as rien d'autre que ce prompt.

**À lire** : la fiche en entier ; `_bmad-output/planning-artifacts/epic-66-administration-menu.md` ; `CLAUDE.md` (§ *Review Iteration Rule*, *Test Locally First*, *Recette visuelle*) ; `frontend/src/routes/(app)/+layout.svelte` ; `frontend/tests/e2e/sidebar-navigation.spec.ts` ; les issues #852, #959, #851, #247 (`gh issue view N --json title,body`, lecture seule).

**Axes (déclare pour chacun : exercé / non exercé, et ce que tu as fait)** :
1. Chaque affirmation « lue au code » de la fiche (numéros de ligne, noms, comptes) : vérifiée au code, citée.
2. Cohérence interne des décisions D1 à D9 et des AC : trous, contradictions, AC invérifiables ou ambigus.
3. Couverture des issues : tout l'attendu de #852 et #959 (dont les deux « trous de couverture E2E » de #959) est-il dans un AC ?
4. **Specs E2E existantes** : greppe `frontend/tests/e2e` (tout `nav-group-`, `nav-link-`, `kesh:sidebar`, `summary`, `details`, `AxeBuilder`) : une spec que l'accordéon casse et que la fiche n'a pas listée ? Le défaut passe de deux groupes ouverts à un.
5. Mécanique : D3 (clic contrôlé avec `preventDefault` sur `<summary>`, `open` dérivé) fonctionne-t-elle avec Svelte 5 et `<details>` (clavier, focus, rechargement) ? D2 (choix sur la page, groupe de la page, choix mémorisé, remise à zéro au changement de chemin) : cas limites (retour sur un chemin déjà visité, page hors groupe, SSR, `page.url.pathname` réactif).
6. Rétrocompatibilité du stockage (D5) : cas oubliés ; piège `toggle`/#424 réintroduit ?
7. Accessibilité (D7, AC6) : la vérification proposée est-elle réalisable (`ariaSnapshot`, axe sur `<summary>`) ?
8. Tests de lecture du source du layout (`mcp-sends-i18n.test.ts`, `mcp-prompts-i18n.test.ts`, `i18n-keys.test.ts`, `npm run lint-i18n-ownership`) : la fiche les préserve-t-elle ?
9. **Manuel** : `docs/manual/fr/` contre le code, **PDF compris**, aplati avant de greper : `pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '` ; autre phrase du manuel (ou du PDF) qui décrit le menu et deviendrait fausse ? Les manuels DE/EN/IT existent-ils ?
10. Taille et risque de la story au regard de D6 de l'epic (sort si non close le lundi 2026-11-30 au soir) ; règle de splitting ; gates (CLAUDE.md).
11. Respect des règles du dépôt : aucun chemin de la station dans la fiche, aucun mot-clé de fermeture, `Status` et `sprint-status.yaml`.

**Interdits** : n'écris aucun fichier (ni la fiche, ni dans `/tmp`), n'exécute aucune commande qui écrit : pas de `gh` en écriture, pas de `scripts/prepare-release.sh`, `docker`, `sed -i`, `git commit/checkout/reset`, `npm install`. Pas de `pkill`/`killall`/`kill`.

**Rends** : une liste de findings, chacun avec sévérité (CRITICAL, HIGH, MEDIUM, LOW), preuve (fichier:ligne ou extrait), correctif proposé ; un décompte par sévérité ; la liste des axes exercés / non exercés. Un « 0 finding » sans cette liste ne compte pas. Pour tout finding CRITICAL/HIGH qui affirme une absence, vérifie par `grep -nF`.
