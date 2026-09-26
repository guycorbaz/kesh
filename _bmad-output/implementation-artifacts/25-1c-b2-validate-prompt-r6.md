# Prompt — revalidation R6, Story 25-1c-b2 (fiche réécrite)

*Versionné le 2026-09-26. **Une lentille** (Opus), contexte frais, passe **complète** : la fiche a été
réécrite en T0 contre le texte actuel (commit `ecaeadb8`), après la revalidation R5 « dérive » qui avait
remonté 1 CRITICAL et 4 HIGH.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-1c-b1-journal-audit-ecran` — elle porte la 25-1c-b1
**implémentée** (l'écran, revue close) et cette fiche. Ta fiche :
`_bmad-output/implementation-artifacts/25-1c-b2-journal-audit-textes.md` — lis ses AC 1 à 9 (réécrits),
ses Dev Notes et l'entrée « réécriture T0 » de son Change Log. Contexte : la fiche de la b1
`25-1c-b1-journal-audit-ecran.md` (ce que l'écran fait réellement), le code de l'écran
(`frontend/src/routes/(app)/audit-log/+page.svelte`, `+page.ts`, `+layout.svelte`), la route
(`crates/kesh-api/src/routes/audit_log.rs`), les arbitrages de l'epic
(`_bmad-output/planning-artifacts/epic-25-vague1-suite.md` — **ne pas les contester**).

## Ce que tu vérifies

1. **Chaque ligne de l'inventaire de l'AC 3** : le texte cité est-il bien à cette ligne (± quelques
   lignes), et le remplacement est-il juste (accords, sens) ? **Rejoue toi-même le grep lexical et une
   lecture par le concept** sur `docs/manual/fr/*.tex`, `README.md` **et les PDF aplatis**
   (`pdftotext <pdf> - | tr '\n' ' ' | tr -s ' '`, vers le scratchpad) : un site oublié ? un site classé
   « déjà conforme » ou « autre sens » à tort ?
2. **L'AC 4** : l'encadré actuel porte-t-il bien ces quatre énoncés ? La prescription de remplacement
   est-elle **vraie contre le code** — la dévalidation est-elle bien la seule voie qui fait disparaître une
   écriture (cherche les appelants de la suppression d'écriture : `delete_in_tx`, `enforce_immutability`,
   `journal_entry.deleted`) ?
3. **L'AC 5** : chaque comportement décrit de l'écran est-il **vrai contre le code de la b1** (menu, rôles,
   jours UTC, plage inversée, numéro vidé au changement de type, action historique, export plafonné, rien
   n'est journalisé) ? Un comportement de l'écran que le texte ne dit pas et qui laisserait l'utilisateur
   devant une liste vide ou un refus inexpliqué ?
4. **L'AC 6 et 7** : les phrases citées existent-elles ? `:1782` « 5 rôles », `:1803`, `README.md:220`,
   le CHANGELOG `[0.12.1]`. Une autre phrase des manuels, du README, du CHANGELOG, de `website/` ou de
   `docs/` dit-elle encore que l'écran du journal d'audit n'existe pas ou « reste à venir » (#378) ?
5. **L'AC 2** : la clé et ses trois cellules ; une autre valeur de catalogue qui nomme le journal
   autrement ?
6. **Faisabilité** : le développeur peut-il implémenter sans deviner ?

## Ce que tu rends

- Findings : sévérité (CRITICAL / HIGH / MEDIUM / LOW), endroit exact de la fiche, **preuve** (commande et
  résultat, texte lu), correction.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.**

## Interdits

⛔ N'écris aucun fichier du dépôt ; n'exécute aucune commande qui écrit dans le dépôt ou une base —
`scripts/*.sh`, `make`, tout `git commit`/`add`/`checkout`/`reset`/`stash`/`switch`, `sqlx migrate`,
`cargo test`, `npm run`, `npx`. Lecture, `grep`, `git log`/`show`/`diff`, `gh issue view`, `pdftotext`
(vers `/tmp/claude-1000/-home-gcorbaz-devel-kesh/ca5ce2e2-67a3-4eeb-817f-c2de35620a1c/scratchpad/`) seulement.
