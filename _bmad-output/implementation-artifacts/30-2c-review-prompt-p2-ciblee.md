# Prompt — passe ciblée P2 de la revue de code, Story 30-2c (document joint et consulté)

*Versionné le 2026-10-20. **Passe ciblée** (`CLAUDE.md` § *La passe ciblée*) : **une seule lentille**, contexte frais,
**Haiku**, braquée sur le **seul commit de remédiation de code** de la revue P1. Lecture seule stricte.*

Dépôt `/home/gcorbaz/devel/kesh-30-2`, branche `story/30-2c`. Le diff à relire, et lui seul :

```sh
git -C /home/gcorbaz/devel/kesh-30-2 diff a4bffcac 9ad839ce
```

`a4bffcac` = l'état revu en P1 ; `9ad839ce` = le **commit de remédiation de code** (`fix(302c): remédiation de la
revue P1 …`, dix fichiers : code, tests, `docs/api-external.md`, `docs/manual/fr/user-manual.tex`). Les commits
suivants ne portent que les PDF régénérés, la fiche, le registre, `sprint-status.yaml` et ce prompt : seuls les PDF
entrent dans la passe (axe 6), pour vérifier qu'ils disent ce que dit le `.tex`.

Rapports qui l'ont motivée : `/home/gcorbaz/devel/kesh-gate-logs/302c-review-p1-B.md`, `302c-review-p1-E.md`,
`302c-review-p1-A.md`. La remédiation est décrite à la fiche
`_bmad-output/implementation-artifacts/30-2-documents-factures-fournisseur.md` (Dev Agent Record, § *Annotations de la
revue de code P1* ; entrée « Pass 1 review » du Change Log) et au registre `epic-30-choix-autonomes-302.md`
(annotations datées de **C-30-2-46** et **C-30-2-51**, choix neufs **C-30-2-52 à 54**). Règles : `CLAUDE.md`
(§ *La passe ciblée*, § *Une passe qui ne déclare pas ses axes exercés ne compte pas*, § *Haiku-specific guardrails*,
§ *Le prompt d'une passe doit NOMMER le manuel*) ; `docs/agents-orchestration.md` § 6.

⚠️ **Lis le diff d'un seul tenant ci-dessus, pas une suite de commits** ; une ligne citée se vérifie sur le **fichier
final** (`sed -n` / `grep -nF`), jamais sur un numéro de ligne de hunk.

## Lentille T — régression de la remédiation P1

Pour **chaque ligne ajoutée** du diff qui affirme un fait (comportement, texte, chemin, nombre), vérifie-le au code
par `grep -nF` ou `sed -n` et **copie la sortie**. Cherche surtout ce que le patch a pu casser autour de lui. Axes,
tous à exercer :

1. **La relecture après un refus** (`frontend/src/lib/features/imported-supplier-invoices/stale-refusal.ts`,
   `AttachDocumentPicker.svelte` : `loadRows`, `attach` ; `frontend/src/routes/(app)/supplier-invoices/[id]/+page.svelte` :
   `reloadAfterRefusal`, `attach`, `detach`). `isStaleDocumentRefusal` ne reconnaît-elle que `404` et `409` ? La
   fiche relit-elle la facture **avant** de relancer l'erreur au sélecteur ? Quand la facture relue a un document, le
   sélecteur se ferme et `notifyWarning` reçoit le message : l'erreur n'est-elle **pas** aussi relancée (sinon le
   sélecteur démonté afficherait un message mort) ? Quand elle n'en a pas, l'erreur est-elle bien relancée, et le
   sélecteur relit-il alors sa liste ? Un échec de `load()` pendant la relecture peut-il faire passer la fiche dans
   `errorMsg` (qui remplace toute la fiche) ou laisser `invoice` à `null` ? `loadRows` remet-elle `loadError` à vide
   sur succès, et `loading` reste-t-il cohérent à la seconde lecture ? Le **déplacement** du bloc `detachError`
   **hors** du `{#if invoice.sourceDocument}` : s'affiche-t-il encore sur la fiche d'une facture sans document après
   un `404`, et ne reste-t-il pas affiché à tort après un rattachement réussi (`detachError = ''` dans `attach`) ?
   Après un rattachement réussi qui suit un refus du détachement, ou après la fermeture du sélecteur, un message périmé
   peut-il rester ?
2. **Les gardes du double clic** (`if (detaching) return;`, `if (attaching) return;`) : placées **avant** toute
   remise à zéro du message et avant `confirm` ? Un refus laisse-t-il `detaching`/`attaching` à `false` (le bouton
   redevient-il actif) ? L'affirmation des commentaires — « sous jsdom, un second clic est parti malgré `disabled` »
   — est-elle cohérente avec le test (`supplier-document-page.test.ts`, tests « double clic ») et avec le journal de
   la mutation M-c21/M-c25 (`/home/gcorbaz/devel/kesh-gate-logs/302c-mutP1-M-c21.log`, `-M-c25.log`) ?
3. **Les tests Vitest neufs** (`supplier-document-page.test.ts`, sept tests « revue P1 », l'aide `apiError`, le mock
   `notifyWarningMock`, la **remise à zéro des mocks** dans `beforeEach`) : chaque test établit-il ce que son titre
   nomme, sans passer à vide (une `waitFor` sans assertion qui puisse échouer, un `findBy` sur un élément présent
   avant le geste) ? La remise à zéro (`mockReset`) d'un mock déclaré dans une fabrique `vi.mock` casse-t-elle un
   test existant qui comptait sur une implémentation par défaut (`listImportedMock`, `fetchDocMock`) ? Les mutations
   M-c19 à M-c25 meurent-elles **sur le test qui les nomme** (journaux `302c-mutP1-M-c*.log`, synthèse
   `302c-mutations-p1.log`, `302c-mutations-p1-bis1.log`, `302c-mutations-p1-bis.log`) ?
4. **Les tests Rust neufs** (`crates/kesh-db/tests/supplier_document_attachment.rs` : extension de
   `entry_lookup_covers_purchase_and_settlement`, test `cancelled_invoices_take_a_document_and_the_detached_settlement_loses_it`).
   Les états posés en SQL (lien laissé sur un document `to_complete` ; document d'une **autre société** lié à notre
   facture) sont-ils atteignables malgré les contraintes (`grep -n "imported_si" crates/kesh-db/test-schema/0001_schema_squash.sql` :
   unicité de `supplier_invoice_id`, clé étrangère simple, `CHECK`) — et le test les pose-t-il dans un ordre que
   l'unicité admet ? M-c17, M-c18 et M-c26 meurent-elles sur l'assertion qui les nomme
   (`302c-mutP1-M-c17.log`, `-M-c18.log`, `-M-c26.log`) ? `supplier_invoices::cancel` d'une facture **payée** :
   l'assertion « l'écriture d'achat montre encore le document » est-elle vraie au code (`supplier_invoices.rs`,
   `cancel_in_tx`, colonnes vidées) ?
5. **Doc-comments, registre, cycle d'interblocage** : le doc-comment d'`attach_in_tx`
   (`crates/kesh-db/src/repositories/imported_supplier_invoices.rs`), celui de `attach_source_document` et de
   `detach_source_document` (`crates/kesh-api/src/routes/imported_supplier_invoices.rs`) et le point (vi) de
   `crates/kesh-api/tests/audit_route_registry.rs` nomment désormais la confirmation d'un lot de paiement : est-ce
   vrai au code (`crates/kesh-db/src/repositories/payment_batches.rs`, `confirm_batch`, boucle de `pay_in_tx` ;
   `crates/kesh-api/src/routes/payment_batches.rs`, `retry_on_deadlock`) ? Reste-t-il un site qui présente le cycle
   avec la **seule** acceptation d'un lot (`grep -rnE "acceptation d.un lot de rapprochement" crates docs CHANGELOG.md README.md`) ?
   La réfutation de `bank_imports::delete` (C-30-2-46, annotation) tient-elle (`delete_unreconciled_in_tx`) ? Le
   doc-comment d'`attach_in_tx` sépare-t-il bien le **mesuré** du **raisonné** ? Le doc-comment du test
   `attach_waits_for_an_uncommitted_detach_then_succeeds` (preuve redéfinie de M-c11) dit-il vrai sur la sonde
   (`attendre_une_requete_en_cours`, ses motifs) ? Les angles morts de C-30-2-52 (relecture après `COMMIT`,
   non-idempotence) sont-ils exacts (le refus rendu à un rattachement renvoyé est-il bien
   `IMPORT_NOT_PENDING_COMPLETION`, et non `SOURCE_DOCUMENT_ALREADY_ATTACHED` — `attach_in_tx`, étape 1) ?
6. **Manuel, PDF et `api-external.md`** : `docs/manual/fr/user-manual.tex` (phrase du règlement d'une facture payée
   puis annulée) et `docs/api-external.md` (même fait ; deux phrases « relisez la facture avant de conclure » ;
   paragraphe des interblocages) disent-ils vrai au code ? Le **PDF** dit-il la même chose que le `.tex` — contrôle
   **aplati vers la sortie standard seulement** :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oF "payée puis annulée"` et
   `… | grep -oF "L'écriture d'achat d'une facture annulée"` (apostrophe typographique possible : essayer aussi `’`) ;
   le PDF est-il **postérieur** au `.tex` (`git log -1 --format=%ci -- <fichier>`) ?
7. **Fiche et comptes rendus** : la remédiation a-t-elle **réécrit** une entrée passée (Change Log, registre, Dev Agent
   Record) au lieu de l'**annoter** (interdit) ? Les décomptes écrits (sept Vitest, un test Rust neuf et un étendu,
   mutations M-c17 à M-c26 « dix jouées, dix tuées », dix fichiers au commit, gates et E2E) se retrouvent-ils dans
   le dépôt et dans les journaux cités (`302c-gate3-*.log`, `302c-e2e-p1.log`) ? Recompte, aux deux bornes :
   `git show a4bffcac:"frontend/src/routes/(app)/supplier-invoices/[id]/supplier-document-page.test.ts" | grep -cE "^\s*it\("`
   et le fichier final ; idem `grep -c "#\[sqlx::test" crates/kesh-db/tests/supplier_document_attachment.rs`.
8. **Mots-clés de fermeture** : `git -C /home/gcorbaz/devel/kesh-30-2 log a4bffcac..HEAD --format=%B | grep -niE "\b(close[sd]?|fix(e[sd])?|resolve[sd]?):?\s+#[0-9]+"` doit être vide.

**Haiku — garde-fous** (`CLAUDE.md`) : tout finding qui affirme l'**absence** d'un patch ou la **présence** d'un
anti-pattern s'appuie sur une sortie `grep -nF` copiée, sur le **fichier final** et non sur une ligne d'un hunk ; un
livrable que la fiche prescrit ailleurs qu'à ce commit n'est pas un défaut du commit ; ne signale que ce que le diff
**affirme** et qui est faux, ou ce qu'il casse.

## Ce que tu rends

Findings numérotés `T1…` avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, fichier et ligne, **preuve** (commande
exécutée et sortie copiée), correction proposée. ⛔ **La liste des axes exercés ET non exercés** (1 à 8 ci-dessus) — un
« 0 finding » sans elle ne compte pas comme passe. Rapport **écrit d'un seul jet**, un seul fichier, à la fin, sans
brouillon ni fichier intermédiaire : `/home/gcorbaz/devel/kesh-gate-logs/302c-review-p2-T.md`. Dernier message : le
chemin du rapport, le bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport — ni dans `/tmp`, ni ailleurs, pas même temporairement. Lecture seule
stricte : **aucun `git checkout`, `git switch`, `git checkout --detach` ni aucune autre forme** ; aucune commande qui
écrit, compile ou exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`,
`scripts/mem-guard.sh`, `scripts/regen-test-schema.sh`), `make`, `latexmk`, `xelatex`,
`git commit`/`add`/`stash`/`fetch`/`pull`/`reset`/`rebase`/`tag`/`push`, `cargo`, `npm`, `npx`, `node`, **`python3`,
`python`, `perl`, `ruby` et tout autre interpréteur**, **`sed -i`**, `docker`, **`mariadb`, `mysql` et toute requête
SQL**, `bash` sur un script du dépôt ou de `/home/gcorbaz/devel/kesh-gate-logs/`, `gh` en écriture. `pdftotext` vers
la **sortie standard** seulement, jamais vers un fichier ; aucune redirection `>` vers un fichier. Aucune donnée réelle
de Guy ne se lit ni ne se cite (rien de `~/kesh-test/`). Autorisés : lecture, `grep`, `sed -n`, `awk` en lecture,
`git log`/`show`/`diff`/`blame`, `pdftotext … -`.
