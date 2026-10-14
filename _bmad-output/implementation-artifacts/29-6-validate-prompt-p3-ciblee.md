# Prompt — validation P3 (passe ciblée), Story 29-6 (la réconciliation ne propose plus une facture qu'un trop-perçu de quelques centimes fait refuser)

*Versionné le 2026-10-14. **Passe ciblée** (`CLAUDE.md` § *La passe ciblée*) : **une seule lentille**, contexte frais,
**Haiku**, braquée sur **le seul diff de la remédiation P2** — pas sur la story entière. Rotation (D6, rétro 25) :
Haiku est réservé aux passes ciblées ; P1 était Sonnet, P2 Opus, la remédiation P2 a été écrite par Opus.*

Dépôt `/home/gcorbaz/devel/kesh-29-6`, branche `story/29-6`, code au sol `fcd2ae75` (kickoff de l'Epic 29 ; la story
n'a encore modifié **aucun** fichier de code — seulement des fichiers de planning).

**Objet unique** — la remédiation P2, commit `e20a418e`, lu **aplati** en un seul diff :

```sh
git -C /home/gcorbaz/devel/kesh-29-6 diff 2410c8a2 e20a418e
```

(Un seul diff, deux bornes : ne lis pas une séquence de commits, l'indexation des numéros de ligne s'y perd.) Il touche
quatre fichiers : la fiche `_bmad-output/implementation-artifacts/29-6-trop-percu-centimes.md`, le registre
`_bmad-output/implementation-artifacts/epic-29-choix-autonomes-29-6.md`, `_bmad-output/implementation-artifacts/sprint-status.yaml`
et `_bmad-output/planning-artifacts/epic-29-preparer-cloture.md`.

Ce que la remédiation devait corriger : les rapports P2, `/home/gcorbaz/devel/kesh-gate-logs/29-6-validate-p2-R.md`
(0/0/2/9) et `/home/gcorbaz/devel/kesh-gate-logs/29-6-validate-p2-F.md` (0/0/2/7), et l'entrée « Validation P2 » du
Change Log de la fiche (dernière entrée). Issue de l'angle mort : `gh api repos/guycorbaz/kesh/issues/670`.

## Le motif que tu chasses

*La sévérité se déplace vers ce qu'on vient d'écrire* : sur les Epics 22 et 23, la plupart des défauts des passes
tardives étaient **introduits par le patch précédent**. Ta question n'est pas « la story est-elle bonne ? » mais :
**la remédiation P2 a-t-elle introduit un défaut, ou laissé un résidu de ce qu'elle corrigeait ?**

## Axes (et seulement eux)

1. **Fidélité aux findings** : pour chacun des vingt findings P2 (R1 à R11, F1 à F9), la fiche porte-t-elle la
   correction annoncée par le Change Log ? Un finding déclaré traité et non traité, ou traité à moitié, est un finding.
2. **Exactitude au code des faits neufs** — chaque numéro de ligne et chaque affirmation ajoutés par le diff, revérifiés
   au sol par `sed -n` ou `grep -nF` sur le fichier **final** (pas sur le diff) :
   `crates/kesh-api/src/routes/reconciliation.rs` (`:640-652`, `:1915`, `:1930-1931`, `:3256-3266`),
   `crates/kesh-db/src/repositories/reconciliation.rs:271` (`fetch_all`), `crates/kesh-db/src/repositories/bank_accounts.rs:1025`,
   `frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte` (`:109`, `:205-207`, `:249`, `:270`,
   `:280`, `:320-326`, `:371-397`), `ReconciliationProposals.test.ts:108-123`,
   `frontend/src/lib/features/bank-import/DeleteBankImportAction.test.ts:191-194`,
   `frontend/tests/e2e/reconciliation-deleted-import.spec.ts:12-14` et `:153-154`,
   `frontend/tests/e2e/reconciliation-unreject.spec.ts:146-150` (et les `data-testid` qu'il cite),
   `frontend/node_modules/svelte/src/internal/client/dom/elements/events.js` (condition `event.target === current_target`),
   `docs/manual/fr/user-manual.tex` (`:1690-1694`, `:2029`, `:2050`, `:2057`, `:2910-2911`, label
   `sec:clore-compte-bancaire`).
3. **Cohérence interne après le patch** : l'ancien texte (« ne peut qu'être rejetée », `reject-only`, « sous les
   boutons », « pour *Rejeter* seulement », « sans quoi la transaction reste en attente », « inatteignable », « aucun test
   ne la tue », « 11 mutations », « C-29-6-1 à 15 ») survit-il **hors** des citations explicites et des entrées datées ?
   Les nombres neufs (13 mutations, 19 choix, 0/0/4/16, vingt findings) se recomptent-ils depuis la source ? Le tableau
   AC8 (M10 à M13) et les tests Vitest prescrits se correspondent-ils un à un (chaque mutation tuée par un test écrit) ?
4. **Les mutations neuves sont-elles tuables par les tests prescrits** : M12 (garde d'`onAccept` retirée) par le test
   `fireEvent.click` sur *Accepter* grisé ; M13 (*Rejeter* grisé) par l'assertion `disabled === false` ; M10 par
   l'attribut seul. Raisonne sur le code au sol et l'encadré de l'AC8 (jsdom + délégation Svelte 5) — **sans rien
   exécuter**.
5. **Les textes prescrits neufs disent-ils vrai** : l'indication dans les quatre locales (« ne peut pas être
   acceptée »…, termes de l'écran : `reconciliation-labels-no-candidate`, `-actions-accept` dans les quatre
   `crates/kesh-i18n/locales/*/messages.ftl`), les phrases prescrites au manuel (`:2029`, `:2050`, `:2057`, `:2910`) et
   au CHANGELOG : l'une d'elles dit-elle encore, ou dit-elle désormais, que le rejet **règle** la transaction ou la sort
   de l'attente (contraire à #670) ? En nomme-t-elle *Affecter manuellement* / *Éclater* comme gestes d'un paiement de
   client (contraire à la FAQ `:2910`) ? **Manuel** : les contrôles PDF prescrits (positifs, négatifs, normalisation
   `s/’/'/g`) sont-ils exécutables tels qu'écrits ? Contrôle le PDF actuel, aplati et normalisé, **vers la sortie
   standard seulement** :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | sed "s/ﬀ/ff/g;s/ﬁ/fi/g;s/ﬂ/fl/g;s/ﬃ/ffi/g;s/’/'/g" | grep -o "…"`.
6. **Les entrées datées** : le diff a-t-il **réécrit** une entrée datée au lieu de l'annoter (Change Log P1 de la fiche,
   corps des choix C-29-6-10 à 12 du registre, commentaire du kickoff d'`epic-29` dans `sprint-status.yaml`, ligne
   « Étendue à la validation P1 » du fichier d'epic) ? Comparer `-` et `+` du diff : une ligne datée retirée sans être
   restituée est un finding. Le commentaire du kickoff porte-t-il de nouveau « (29-2..4) », suivi d'une annotation
   datée ?
7. **Le choix nouveau** (C-29-6-16 à 19) : chacun a-t-il contexte, retenu, écartées, réversible ? L'argument de
   C-29-6-16 (la FAQ interdit *Affecter manuellement* / *Éclater* pour un paiement de client) est-il exact au manuel ?
   Le placement de C-29-6-18 (dernier enfant du `div` des boutons, `flex-wrap items-center`) contredit-il la fiche
   29-3 (`/home/gcorbaz/devel/kesh-29-3/_bmad-output/implementation-artifacts/29-3-meilleure-proposition.md:286-290`) ?

Hors périmètre : la démonstration de la borne, AC1 à AC6, le découpage — tenus par P1 et P2. N'y reviens que si le
diff P2 les touche.

## Sévérité

Une prescription fausse au code, une mutation déclarée tuée qui survivrait au test prescrit, un texte prescrit qui
promet ce que l'écran ou le code ne fait pas, une entrée datée réécrite : au moins **MEDIUM**. Un numéro de ligne
décalé sans effet sur le sens, une coquille : **LOW**.

## Ce que tu rends

Findings numérotés `P3-1`, `P3-2`… avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`, **preuve**,
correction proposée. ⛔ **Toute affirmation de présence ou d'absence se prouve par `grep -nF "<chaîne>" <fichier>` sur
le fichier FINAL, sortie copiée** — jamais par le diff seul (le `-F` est obligatoire : les métacaractères de `Vec<i64>`,
`:640-652`, `*Rejeter*` font mentir un grep en regex). Un finding CRITICAL ou HIGH « absent » ou « non appliqué » sans
cette sortie ne sera pas retenu.

⛔ **La liste des axes exercés ET non exercés** (1 à 7), avec pour chacun ce qui a été lu ou greppé — un « 0 finding »
sans elle ne compte pas comme passe, et un axe « examiné rapidement » se déclare non exercé.

Rapport dans **`/home/gcorbaz/devel/kesh-gate-logs/29-6-validate-p3-ciblee.md`** ; dernier message : le chemin du
rapport, le bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits — lecture seule stricte

⛔ Le **seul** fichier que tu écris est `/home/gcorbaz/devel/kesh-gate-logs/29-6-validate-p3-ciblee.md`. **Aucune
redirection de sortie vers un fichier** ailleurs, `/tmp` compris. Aucune commande qui écrit, compile ou exécute :
`scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh`), `make`,
`latexmk`, `xelatex`, `git commit`/`add`/`checkout` **sous aucune forme** (`--detach` compris)/`switch`/`restore`/
`stash`/`fetch`/`pull`/`reset`/`rebase`/`tag`/`push`/`worktree`, `cargo`, `npm`, `npx`, `node`, `vitest`,
`playwright`, **tout interpréteur** (`python3`, `python`, `perl`, `ruby`, `bash`/`sh` sur un script, `awk` qui
écrit), `sed -i`, `docker`, **`mariadb`/`mysql`** et toute requête SQL, `gh` en écriture. Rien de `~/kesh-test/`
(relevé réel de Guy) ne se lit ni ne se cite. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`/`blame`,
`gh api` en lecture, `pdftotext` vers la sortie standard.
