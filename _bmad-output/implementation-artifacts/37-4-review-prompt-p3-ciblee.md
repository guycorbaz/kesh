# Prompt de revue de code P3 ciblée — story 37-4

Passe **ciblée** (`CLAUDE.md` § *La passe ciblée*), une seule lentille, contexte frais, braquée sur la remédiation P2 :
worktree `<worktree>/kesh-s37-4` (branche `story/37-4-rapports-en-devise`),
`git show d1d7164d`. N'écris nulle part ailleurs, sauf journaux dans `$GATE_LOGS/s37-4/review-p3/`.

À lire : la fiche 37-4 (AC, entrées « revue de code P1 » et « P2 » du Change Log).

Correctifs de la P2 à relire : `write_fx_line` et `FX_RAISE_MM` dans `crates/kesh-report/src/pdf.rs` (seconde ligne en
devise remontée, avance totale inchangée), troncature des noms en devise à 34 caractères, `formatCurrencyFigure` dans
`reports.api.ts` utilisé par `TrialBalanceView.svelte`, test `projets_en_chf`.

Cherche : défauts introduits (la remontée chevauche-t-elle la ligne du mouvement au-dessus, un report « À reporter »,
un saut de page, la rupture d'exercice ? l'avance totale est-elle vraiment inchangée partout où la seconde ligne est
écrite — grep des appels ? les fixtures mono-devise restent-elles identiques ? `formatCurrencyFigure` est-il employé
partout où un montant en devise s'affiche à l'écran — grep — et `formatCurrencyAmount` garde-t-il son comportement ?
les tests rougissent-ils sans leur correctif ?). Si possible, régénère un PDF de grand livre et de balance d'un compte
en devise et regarde-le. Déclare les axes exercés / non exercés.

Phase 2 : corrige, avec tests ; gate ciblé (fmt, clippy workspace, nextest `-p kesh-report` et binaires touchés ;
Vitest et `npm run build` si le frontend est touché) par `$GATE_LOGS/heavy.sh scripts/mem-guard.sh
<commande>` ; commit « fix(37-4) : revue de code P3 — … » (aucun mot-clé de fermeture, **aucune mention d'IA**, aucun
`GRANT`). Change Log « 2026-11-16 — revue de code P3 ciblée (<modèle>) : <décompte> ». Aucun push. Si tu ne corriges
rien : aucun commit, aucun gate.

Rapport : findings `37-4-CR-P3-N` (sévérité, fichier:ligne, constat, correctif), axes, gates (chiffres), décompte.
