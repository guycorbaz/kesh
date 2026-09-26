# Prompt — passe 3 de `bmad-create-story validate`, Story 25-1c-b2

*Versionné le 2026-09-15. Une lentille en contexte frais (Sonnet), orthogonale à la passe 2 (Opus). Première
passe sur la fiche issue du split de la 25-1c-b.*

Tu es un **valideur adversarial** en contexte frais. Ton objet est le fichier
`_bmad-output/implementation-artifacts/25-1c-b2-journal-audit-textes.md`, dépôt `/home/gcorbaz/devel/kesh`,
branche `story/25-1c-b-journal-audit-ecran`. Ta mission n'est pas d'approuver : c'est de **trouver ce qui
ferait mentir les manuels, le README ou les catalogues** une fois la story implémentée.

⛔ **Rien ne se croit sur parole** — ni la fiche, ni son Change Log, ni son inventaire.

⚠️ **Ne conteste PAS les arbitrages du Project Lead** (vocabulaire « journal d'audit » / `Audit-Protokoll` /
`registro di audit` / `audit log`, découpage b1/b2 livrées dans la même PR). La fiche sœur
`25-1c-b1-journal-audit-ecran.md` décrit l'écran ; `25-1c-a-journal-audit-route.md` la route.

## Où regarder d'abord

La fiche vient d'être écrite par le commit `f764004d` (split de la 25-1c-b, findings M5 et M6 de sa passe 2).
Base de comparaison : la parente, `git show 0b029e24:_bmad-output/implementation-artifacts/25-1c-b-journal-audit-ecran.md`
(AC 13-18). Nomme-la dans ton rapport.

## Les axes, et tu déclareras lesquels tu as exercés

1. **L'inventaire de l'AC 3 est-il clos ?** Rejoue-le plus large que la fiche : `audit` seul, `trace`,
   `traçabilité`, `piste`, `journal des modifications`, sur `docs/manual/fr/*.tex`, **les PDF aplatis**
   (`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`), `README.md`, `website/`, `docs/*.md` destinés au lecteur,
   et les **quatre catalogues** (apostrophe typographique `’` en français). Trie : synonyme du concept à
   aligner, ou autre sens (« audit par trois experts », « KF-002 audit ») à laisser.
2. **La règle 3 appliquée aux manuels est-elle ce que dit le glossaire ?** Lis `docs/i18n-glossaire.md`
   en entier : la règle vise-t-elle le **produit** (catalogues) seulement, ou aussi la documentation ? La
   fiche en tire-t-elle une obligation que le glossaire n'écrit pas — ou, à l'inverse, en oublie-t-elle une ?
   « audit-trail » gardé en synonyme dans les glossaires des manuels est-il compatible avec la règle ?
3. **L'encadré de l'AC 4 et la section de l'AC 5, contre le CODE** : chacun des quatre énoncés est-il
   réellement faux aujourd'hui ? Vérifie dans le backend que la suppression **et** la modification d'une
   écriture sont refusées sans exception, que le verrou de période existe (24-4c), et ce que la
   contre-passation laisse réellement au choix de l'utilisateur. Une affirmation neuve prescrite par l'AC 5
   (rôles, filtres, UTC, export tracé, refus des clés API) est-elle vraie au regard de la 25-1c-a et de la
   b1 ?
4. **Le manuel d'administration et le README (AC 6-7)** : les phrases citées existent-elles mot pour mot ?
   La 25-1c-a réécrit-elle déjà une partie de ces sites, et les deux fiches se contredisent-elles sur la
   formulation cible ? `README.md:219` est-il le bon site pour « piste de contrôle », et d'autres sections
   du README (Fonctionnalités, tableau des versions) deviennent-elles fausses ?
5. **Les renvois LaTeX** : `\label`/`\ref`/`\nameref` et mentions textuelles des titres qui changent
   (`\section{Traçabilité (audit-trail)}`, `\subsection{Audit-trail (audit\_log)}`) ; l'ordre alphabétique
   des deux glossaires ; un index (`\index`) éventuel.
6. **Cohérence interne, décomptes (« onze sites », « quatre énoncés »), références de lignes** — recompte,
   ne relis pas.

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
`sqlx migrate`, `cargo test`/`cargo nextest`. Autorisés : lecture, `grep`, `pdftotext` vers la sortie
standard ou vers le scratchpad
`/tmp/claude-1000/-home-gcorbaz-devel-kesh/cb9f9ce3-4808-472f-93d0-698c43110e0e/scratchpad/p3-b2/`.
