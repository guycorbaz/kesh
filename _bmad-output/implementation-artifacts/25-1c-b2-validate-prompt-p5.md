# Prompt — passe 5 de `bmad-create-story validate`, Story 25-1c-b2

*Versionné le 2026-09-15. Une lentille en contexte frais (Sonnet), orthogonale à la passe 4 (Opus). La
remédiation de la passe 4 a réécrit la fiche presque entière : la passe porte sur ce qui a changé, et sur
tout ce qui en dépend.*

Tu es un **valideur adversarial** en contexte frais. Ton objet est la fiche
`_bmad-output/implementation-artifacts/25-1c-b2-journal-audit-textes.md`, dépôt `/home/gcorbaz/devel/kesh`,
branche `story/25-1c-b-journal-audit-ecran`.

**Base** : `git diff 637c6508 c5d14f32 -- _bmad-output/implementation-artifacts/25-1c-b2-journal-audit-textes.md`.
Nomme-la dans ton rapport. Le motif mesuré du projet : *la sévérité se déplace vers ce qu'on vient
d'écrire* — sur cette fiche, les passes 3 et 4 ont chacune trouvé un défaut **introduit par la remédiation
précédente**.

⛔ **Rien ne se croit sur parole** — ni la fiche, ni son Change Log, ni les « vérifié » qu'il déclare.
⚠️ **Ne conteste PAS les arbitrages du Project Lead** (vocabulaire, découpage, texte du point 1 de
l'arbitrage du soir). Les deux défauts voisins marqués « EN ATTENTE D'ARBITRAGE » ne sont pas à trancher :
vérifie seulement qu'ils sont exacts.

## Les axes, et tu déclareras lesquels tu as exercés

1. **Chaque ligne neuve des tableaux dit-elle vrai ?** Les cinq sites « concept » de l'AC 3, les sept sites
   « autre sens », les cinq énoncés de l'AC 4, les cellules ✓/✗ de l'AC 2, les deux défauts voisins de
   l'AC 6. Ouvre chaque ligne citée — `.tex` **et** PDF aplati (`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`).
   Un site trié « autre sens » désigne-t-il en fait le journal d'audit ?
2. **Le motif élargi de l'AC 8 et son « reste attendu »** : exécute-le sur les PDF aplatis et le README
   **actuels**, et compare son rendu, ensemble contre ensemble, au tableau de l'AC 3 plus les sites triés.
   Rend-il un site que ni le tableau ni le tri ne couvrent ? Le « reste attendu » après implémentation
   est-il exactement celui qu'annonce l'AC 8 — ou d'autres occurrences légitimes (le mot « piste » dans un
   autre sens, `audit_log` technique) y resteront-elles et le feront échouer à tort ?
3. **Les énoncés neufs de l'AC 5, contre le contrat** : égalité exacte du filtre d'action, identifiant
   exigeant un type, plafond de 10 000 lignes et son message, sens de « société indéterminée ». Lis
   `25-1c-a-journal-audit-route.md` (AC 2, 5, 11) et `25-1c-b1-journal-audit-ecran.md` (AC 5) : chaque
   phrase que la b2 prescrit est-elle ce que le code livrera ? « Auteur dont la société n'a pas pu être
   établie » est-il la définition exacte de `company_id` nul (25-1c-zero) ?
4. **Le tableau des trois sources de l'AC 1** : la citation de l'arbitrage du soir est-elle **mot pour
   mot** celle de `epic-25-vague1-suite.md` ? Les précédents de glossaire cités (`fr-CH:1241`, `en-CH:1184`,
   `de-CH:729`) attestent-ils bien le terme ? La vérification des clés neuves (AC 2) est-elle exécutable
   telle qu'écrite ?
5. **Cohérence interne après réécriture** : décomptes (« dix-sept », « cinq énoncés », « trois cellules sur
   huit », tallies des Change Logs), renvois entre AC, tâches qui exécutent chaque AC (une tâche oubliée ?),
   References. Recompte, ne relis pas.

## Ce que tu rends

- **Les findings**, chacun avec sévérité (`CRITICAL` / `HIGH` / `MEDIUM` / `LOW`), l'endroit exact, **la
  commande ou l'extrait qui l'établit**, et le correctif.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.** Ne qualifie jamais de
  « vérifié » ce que tu n'as pas exécuté.

## Interdits

⛔ **N'écris aucun fichier du dépôt et n'exécute aucune commande qui écrit dans le dépôt ou dans une base
persistante** — nommément `scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`,
`scripts/install-hooks.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make`, `latexmk`, tout
`git commit`/`push`/`checkout`/`add`/`stash`/`reset`/`rebase`, `npm install`, `npm run build`,
`sqlx migrate`, `cargo test`/`cargo nextest`. Autorisés : lecture, `grep`, `git diff`/`git show`,
`pdftotext` vers la sortie standard ou vers le scratchpad
`/tmp/claude-1000/-home-gcorbaz-devel-kesh/cb9f9ce3-4808-472f-93d0-698c43110e0e/scratchpad/p5-b2/`.
⚠️ `grep` est ici `ugrep`, qui refuse les motifs trop complexes (`{0,160}` avec alternatives) : utilise
Python pour les extractions à contexte.
