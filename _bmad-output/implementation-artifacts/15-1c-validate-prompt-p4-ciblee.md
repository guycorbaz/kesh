# Prompt — validation P4 CIBLÉE des specs 15-1c-0, 15-1c-i, 15-1c-ii (une lentille, Haiku)

*Versionné le 2026-10-09. Passe ciblée de fin de boucle (`CLAUDE.md` § « La passe ciblée ») : une seule lentille,
contexte frais, lecture seule, braquée sur **le seul commit de la remédiation précédente**.*

Dépôt `/home/gcorbaz/devel/kesh-15-1c`. La validation P3 (Sonnet ×2) a rendu 0 au-dessus de LOW ; ses LOW ont été
appliqués par le commit **`99280a24`** (fiches dans `_bmad-output/implementation-artifacts/` :
`15-1c-0-groupe-de-lettrage-enrichi.md`, `15-1c-i-ecran-postes-ouverts.md`, `15-1c-ii-lettrage-dans-kesh.md`, plus le
registre `epic-15-choix-autonomes.md`, C-15-1c-24 à 26). Les rapports P3 qu'il remédie :
`/home/gcorbaz/devel/kesh-gate-logs/15-1c-validate-p3-R.md` et `-F.md`.

**Ton périmètre : le diff `git show 99280a24 -- '*.md'`, et RIEN d'autre** (lis-le aplati, une seule fois ; ne lis
pas les commits antérieurs comme diff). Pour chaque hunk, la question est : **ce texte neuf est-il vrai, et
introduit-il un défaut ?** Vérifie contre :

1. le code de `f9b6b199` : `crates/kesh-db/src/repositories/letterings.rs` (`FIND_GROUP_SQL`, `LOCK_LINES_BY_KEY_SQL`,
   `struct LineRow`, `find_group`, `letterable_account`, `group_account_number`), `crates/kesh-api/src/routes/letterings.rs`
   (`LetteringResponse`), `frontend/src/routes/(app)/journal-entries/[id]/+page.svelte` (pied *Total*),
   `frontend/src/lib/shared/i18n-entrees-a-variables.test.ts`, `crates/kesh-i18n/locales/fr-CH/messages.ftl`
   (`error-lettering-concurrent-change`), `docs/manual/fr/*.tex`, `docs/api-external.md` ;
2. le reste de la même fiche : le texte neuf contredit-il un autre paragraphe, un test, une tâche (propagation) ?
3. les fiches amont nommées par le texte neuf (15-1b AC1 pour la transaction ; 15-1b-0 D1 pour `DocumentOwner`).

Ne rapporte **pas** de finding sur un texte que le commit n'a pas touché. ⚠️ Si tu affirmes qu'un texte est absent
ou présent, copie la sortie de `grep -nF` qui le prouve — un « patch non appliqué » non étayé par `grep -nF` sur le
fichier **actuel** ne compte pas. Ne confonds pas « livrable prescrit par la fiche mais absent du code » avec un
défaut : ces fiches ne sont pas développées, le code de leurs livrables n'existe pas encore.

## Ce que tu rends

Findings numérotés (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, preuve (sortie copiée), correction proposée. ⛔ La
liste des **hunks examinés** et de ceux **non examinés** — un « 0 » sans elle ne compte pas. Rapport dans
`/home/gcorbaz/devel/kesh-gate-logs/15-1c-validate-p4-F.md` ; dernier message : le chemin, le bilan, une ligne par
MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou exécute :
`scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`, `git commit`/`add`/`checkout`/`switch`/`stash`/
`fetch`/`reset`/`rebase`, `sqlx`, `cargo`, `npm`, `npx`, `docker`, `gh` en écriture, SQL. Autorisés : lecture, `grep`,
`sed -n`, `git show`/`log`/`diff`, `pdftotext` vers la sortie standard.
