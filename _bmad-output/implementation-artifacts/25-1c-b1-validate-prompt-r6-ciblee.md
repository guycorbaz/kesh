# Prompt — revalidation R6 CIBLÉE, Story 25-1c-b1

*Versionné le 2026-09-26. Une lentille (Haiku 4.5), contexte frais, braquée sur la seule remédiation de
la R5 « dérive » — CLAUDE.md, § « La passe ciblée ».*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-1c-b1-journal-audit-ecran`. Diff à relire, APLATI :
`git diff bc4fee4b 6358639e -- _bmad-output/implementation-artifacts/25-1c-b1-journal-audit-ecran.md`.

La R5 a corrigé : le nombre de clés et d'actions du vocabulaire (138 / 97 au lieu de 133 / 92, et la fiche
ne fige plus ce nombre), et les numéros de ligne de `frontend/src/lib/shared/i18n-keys.test.ts` et de
`frontend/src/lib/shared/i18n-libelle-en-dur.test.ts`.

## Ce que tu vérifies

1. **Chaque valeur corrigée**, contre la source actuelle : recompte `ACTIONS` et `ENTITY_TYPES` dans
   `crates/kesh-api/src/audit_labels.rs`, `grep -c '^audit-log-' crates/kesh-i18n/locales/*/messages.ftl`,
   et chaque numéro de ligne cité (`grep -nF` du symbole dans le fichier de test).
2. **Le symptôme ailleurs dans la fiche** : reste-t-il un « 133 », un « 92 », ou un numéro de ligne de ces
   deux fichiers de test non corrigé ? (`grep -nE "\b(133|92)\b"` sur la fiche, puis tri à la main.)
3. Le raisonnement qui s'appuyait sur ces nombres tient-il encore ?

## Ce que tu rends

- Findings : sévérité, endroit, **preuve** (commande et résultat), correction ; pour tout CRITICAL ou HIGH,
  la commande `grep -nF` exécutée et son résultat.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.**

## Interdits

⛔ N'écris aucun fichier ; n'exécute aucune commande qui écrit dans le dépôt ou une base — dont
`scripts/*.sh`, `make`, tout `git commit`/`add`/`checkout`/`reset`/`stash`, `cargo test`, `npm run`.
Lecture, `grep`, `git diff`/`show` seulement.
