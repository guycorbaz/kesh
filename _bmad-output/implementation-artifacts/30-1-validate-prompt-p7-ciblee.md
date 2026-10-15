# Prompt — validation P7 **ciblée** de la spec, Story 30-1 (la réconciliation règle les factures fournisseur)

*Versionné le 2026-10-15. **Une** lentille, contexte frais, lecture seule. **Haiku**, réservé aux passes ciblées (D6,
rétro 25). La remédiation P6 **retire** une garde (C-30-1-40) et réécrit l'angle mort L1 : elle ôte du comportement,
n'en ajoute pas — la passe ciblée suffit (règle du dépôt, § *La passe ciblée*).*

Dépôt `/home/gcorbaz/devel/kesh-30-1`, branche `story/30-1`. Fiche :
`_bmad-output/implementation-artifacts/30-1-reconciliation-factures-fournisseur.md`. Registre :
`_bmad-output/implementation-artifacts/epic-30-choix-autonomes-301.md` (C-30-1-40 réécrit comme retiré, C-30-1-41
complété). Rapports de la P6 (lecture) : `/home/gcorbaz/devel/kesh-gate-logs/301-validate-p6-{F,R}.md` ; la
remédiation est l'entrée « Validation P6 — retrait de la garde (a) — 2026-10-15 » du Change Log de la fiche.

## Le diff à relire — et lui seul

```sh
git diff d249e528 ad0e4393 -- _bmad-output/implementation-artifacts/30-1-reconciliation-factures-fournisseur.md \
  _bmad-output/implementation-artifacts/epic-30-choix-autonomes-301.md \
  _bmad-output/implementation-artifacts/sprint-status.yaml
```

Travaille sur **ce diff unique**, aplati (pas commit par commit). Ne relis pas la fiche entière ; ouvre le **fichier
final** chaque fois que tu cites une ligne, et le **code** chaque fois qu'une ligne du diff affirme un fait du code.
⚠️ **Tout défaut se cite par sa ligne dans le fichier final** (`grep -nF` sur le fichier à `ad0e4393`), jamais par un
numéro de ligne lu dans le diff.

## La lentille — « la remédiation P6 laisse-t-elle un résidu, ou dit-elle faux sur L1 ? »

**Axe principal 1 — aucun résidu de la garde retirée.** Hors des entrées **datées** du Change Log (« Validation P5 »
et « Validation P6 »), aucune trace ne doit rester, dans la fiche, le registre ou `sprint-status.yaml`, de :
`find_paid_supplier_invoice_by_reference`, `reference_of_paid_supplier_invoice`, `reference_of_paid_invoice`,
`paid_invoice_reference`, `HeldByPaidReference`, la garde (e) de l'étape 3, le critère (c) du rejeu de
`accept_one_rule`, `REGEXP`, T-b48, T-b49, T-b50, M-b31, M-b32, M-b34. Cherche aussi les **formes indirectes** : une
phrase qui parle encore de « deux `notice` », de « quatre raisons » ou « quatre cas » de refus d'une règle, de « la
référence d'une facture déjà payée » comme d'un blocage, de la « forme sans référence » comme seule forme de L1, d'une
précédence entre deux `notice`, d'un test ou d'une tâche qui nomme un numéro retiré. Exemple de commande :

```sh
grep -nE "find_paid|reference_of_paid|paid_invoice_reference|HeldByPaid|REGEXP|T-b4[89]|T-b50|M-b3[124]|quatre (cas|raisons)|deux .notice|sans référence" <fichier>
```

Elle rend des occurrences légitimes (le troisième E2E de l'AC-b12, « sans QR » ; les entrées datées du Change Log) :
trie-les à la main et dis lesquelles tu écartes.

**Axe principal 2 — L1 juste à la lettre contre le code.** Le paragraphe L1 (Dev Notes, § *Angles morts*), le texte
prescrit au manuel (AC-b13, puce « L1, le geste sûr »), la phrase prescrite pour `api-external.md` et pour le CHANGELOG
(AC-b13), « Ce que la ligne ne fait pas » (AC-b5) et la réserve de l'AC-b17 doivent dire **la même chose**, et chaque
fait doit tenir au code :
- les **trois formes** (paiement depuis la fiche, avec ou sans référence ; lot confirmé ; facture payée puis annulée,
  règlement détaché — `crates/kesh-db/src/repositories/supplier_invoices.rs`, doc-comment de `cancel_in_tx`) ;
- le **risque** : la règle, ou une autre facture ouverte du même montant, peuvent être proposées pour ce débit ;
- *Affecter manuellement* et *Éclater* ne proposent que les classes 5, 6, 7 (`frontend/src/lib/features/reconciliation/
  ManualMatchModal.svelte`, `TransactionSplitModal.svelte`) — donc compteraient la dépense deux fois ;
- la transaction `pending` empêche la clôture du compte (`bank_accounts.rs`, `ClosureRefusal::PendingTransactions`) et
  figure au contrôle K2 (`closing_checks.rs`) ;
- l'annulation du paiement depuis la fiche (`supplier_settlement_cancel_blocker`, puis
  `settlement_cancellation::settlement_entry_cancel_blocker`) est refusée sur exercice clos, et **n'est pas** refusée
  pour une facture payée par un lot confirmé ;
- le paiement détaché d'une facture annulée « reste contre-passable » (`journal_entries.rs`, § *Le paiement détaché*) ;
- l'écran ne cite **aucun** numéro d'issue (AC-b11 : la seule `notice` restante) ;
- les lignes de code citées par L1 (`supplier_invoices.rs:1044-1047`, `bank_accounts.rs:1289-1300`,
  `closing_checks.rs:442`, `settlement_cancellation.rs:110`, `journal_entries.rs:1037-1046`,
  `ManualMatchModal.svelte:88`, `TransactionSplitModal.svelte:73`) disent-elles ce que la fiche leur fait dire ?

**Axe 3 — les LOW traités.** Signature de `lock_for_bank_settlement_in_tx` (AC-b6 étape 3 : `amount` ajouté, lu par
(d)) ; AC-b8 (sur le même compte, C1 par le compte ; par l'intervalle, C7 seulement — confronte à la table de l'AC-b10,
ligne `create_batch`) ; registre (annotation P5 placée sous C-30-1-38 ; C-30-1-41 au gabarit ; C-30-1-40 écrit comme
retiré, sans nommer les identifiants retirés).

**Axe 4 — recomptes**, depuis la fiche : 26 AC, 24 tâches, 61 tests nommés (T-a1 à a17 sauf a9 = 16 ; T-b1 à b47 sauf
b43, b44 = 45 ; 7 Vitest), 40 lignes de mutation (37 à exécuter), 41 choix au registre, 37 codes de libellé, modules
30-1a 5 / 30-1b 5. Recompte chacun par une commande copiée.

Un défaut introduit par la remédiation est au moins **MEDIUM** ; un faux rattachement ou un double comptage muet créé
par la 30-1 et **non écrit** à L1 est **HIGH**. Décision du Project Lead non contestable ici : le retrait de la garde et
le report à #671 (ne propose pas de garde neuve).

## Exigences de preuve

Toute affirmation de présence ou d'absence s'appuie sur une sortie `grep -nF` (ou `grep -nE`) **copiée** ; tout fait du
code, sur sa ligne. ⚠️ Un finding qui reproche à la fiche l'**absence** d'un code non encore écrit (la fiche prescrit,
le développement écrit) est une erreur de catégorie. ⚠️ Un « résidu » trouvé dans une entrée datée du Change Log n'est
pas un résidu.

## Lecture seule stricte

⛔ N'écris aucun fichier hors de ton rapport. Aucune commande qui écrit, compile ou exécute : `scripts/*` (dont
**`scripts/prepare-release.sh`**), `make`, `latexmk`, `sed -i`, `perl -pi`, `git commit`/`add`/`checkout` **sous aucune
forme**/`switch`/`restore`/`stash`/`reset`/`rebase`/`tag`/`push`/`worktree`, `cargo`, `npm`, `npx`, `node`, **tout
interpréteur** (`python3`, `python`, `perl`, `ruby`, `bash`/`sh` sur un script), `docker`, **`mariadb`/`mysql`** et toute
requête SQL, `gh` en écriture. `pdftotext`, si tu l'emploies, vers la **sortie standard** seulement
(`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`), jamais vers un fichier. Aucune donnée réelle de Guy. Autorisés :
lecture, `grep`, `sed -n`, `wc`, `ls`, `git log`/`show`/`diff`, `gh api` en lecture.

## Rapport

Dans `/home/gcorbaz/devel/kesh-gate-logs/301-validate-p7-T.md`, **écrit d'un seul jet** par l'outil d'écriture de
fichier (aucune retouche par commande, même sur ton rapport — s'il faut le corriger, réécris-le en entier). Findings
numérotés `T1-…`, sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne` **du fichier final**, preuve copiée,
correction proposée. ⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne compte pas, et un
axe déclaré exercé doit porter au moins une commande copiée. Dernier message : le chemin du rapport, le bilan par
sévérité, une ligne par finding MEDIUM ou plus.
