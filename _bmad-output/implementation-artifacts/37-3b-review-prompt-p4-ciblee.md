# Prompt de revue de code P4 ciblée — story 37-3b

Passe **ciblée** (`CLAUDE.md` § *La passe ciblée*), une lentille, contexte frais, braquée sur la remédiation P3 :
worktree `<dépôt>/.claude/worktrees/kesh-s37-3b` (branche `story/37-3b-ecritures-en-devise-ecrans`),
`git show a646eab8`. N'écris nulle part ailleurs, sauf journaux dans `<station>/kesh-gate-logs/s37-3b/review-p4/`.
La remédiation touche plusieurs modules (kesh-db, kesh-api, frontend) : lis-la **entière**, fichier par fichier.

À lire : la fiche 37-3b (AC, entrées « revue de code P1 », « P2 » et « P3 ciblée » du Change Log).

Correctifs de la P3 à relire : échec passager mémoïsé `TRANSIENT_RETRY_MS` (15 s) dans `rate-lookup.ts` (`get`, `refresh`
qui réessaie toujours, horloge `now` injectable) ; étape « 2 bis » de `journal_entry_conversions.rs` (compte non
reconnu laissé au refus de la création) ; borne `> FX_CHF_MAX` de `complete_entry` ; test de `selectAccount` ;
`docs/api-external.md` ; commentaires.

Cherche : défauts introduits (après 15 s, la requête repart-elle à chaque frappe tant que la panne dure, ou une seule
fois par fenêtre ? un succès après l'échec remplace-t-il bien l'entrée ? le délai se compte-t-il par clé (devise,
mois, genre) ? « Récupérer le mois » contourne-t-il bien le délai ? l'ordre des refus de la conversion est-il celui
que dit l'en-tête du module ?), tests (rougissent-ils sans leur correctif ?), résidus. Déclare les axes exercés /
non exercés. Si tu ne corriges rien : aucun commit, aucun gate.

Phase 2 : corrige, avec tests ; gate ciblé (fmt, clippy workspace, nextest des binaires touchés ; **complet si
`kesh-db` est touché**, par `<station>/kesh-gate-logs/heavy.sh scripts/test-fast.sh --ci` sans `mem-guard.sh`
en plus ; Vitest complet seul puis `npm run build` si le frontend est touché), base `kesh_s37_3b` sur
`kesh-mariadb-dev` (3306), aucun `GRANT` ; commit « fix(37-3b) : revue de code P4 — … » (aucun mot-clé de fermeture,
**aucune mention d'IA**). Change Log « 2026-11-16 — revue de code P4 ciblée (<modèle>) : <décompte> ». Aucun push.

Rapport : findings `37-3b-CR-P4-N` (sévérité, fichier:ligne, constat, correctif), axes, gates (chiffres), décompte.
