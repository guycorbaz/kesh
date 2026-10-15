# Prompt — validation P6 **ciblée** de la spec, Story 30-1 (la réconciliation règle les factures fournisseur)

*Versionné le 2026-10-15. **Une** lentille, contexte frais, lecture seule. **Haiku**, réservé aux passes ciblées (D6,
rétro 25). ⚠️ L'agent de fiche signale que la remédiation P5 ajoute du **comportement** (une requête, une `notice`,
deux raisons de refus, trois points d'appel) : une passe complète est recommandée ; ce prompt existe si l'orchestrateur
garde la passe ciblée.*

Dépôt `/home/gcorbaz/devel/kesh-30-1`, branche `story/30-1`. Fiche :
`_bmad-output/implementation-artifacts/30-1-reconciliation-factures-fournisseur.md`. Registre :
`_bmad-output/implementation-artifacts/epic-30-choix-autonomes-301.md` (C-30-1-40, C-30-1-41 neufs). Rapports de la
P5 (lecture) : `/home/gcorbaz/devel/kesh-gate-logs/301-validate-p5-{F,R}.md` ; la remédiation est l'entrée
« Validation P5 — 2026-10-15 » du Change Log de la fiche.

## Le diff à relire — et lui seul

```sh
git diff 300e1bb8 baee035f -- _bmad-output/implementation-artifacts/30-1-reconciliation-factures-fournisseur.md \
  _bmad-output/implementation-artifacts/epic-30-choix-autonomes-301.md
```

Travaille sur **ce diff unique** (pas commit par commit). Ne relis pas la fiche entière ; ouvre le code chaque fois
qu'une ligne du diff affirme un fait du code.

## La lentille — « la remédiation P5 introduit-elle le défaut suivant ? »

1. **La garde de la référence d'une facture payée** (AC-b3 « La référence d'une facture déjà payée », AC-b5 deuxième
   ligne du tableau, AC-b6 étape 3 (e), AC-b17 (c), AC-b11 `notice`) :
   - est-ce **une** fonction (`find_paid_supplier_invoice_by_reference`), lue par la proposition, l'acceptation d'une
     facture et l'acceptation d'une règle, avec **la même** normalisation et la **même** borne de 6 caractères que l'AC-b1 ?
   - la normalisation SQL prescrite (`UPPER(REGEXP_REPLACE(…, '[[:space:]]', ''))`) est-elle juste en MariaDB 10.11, et
     produit-elle le même résultat que la normalisation Rust (blancs, majuscules) ?
   - le critère « une facture **payée** » est-il trop large — une facture payée **par compte interne**, ou d'un **autre**
     fournisseur que la contrepartie, bloque-t-elle à tort un débit qui porte sa référence ? une facture **annulée** ?
   - précédence avec la `notice` du lot (« la référence prime ») : cohérente avec l'AC-b5 et l'AC-b6 ?
   - sur-blocage : une référence de lot pain.001 (`EndToEndId`, `NtryRef`) peut-elle égaler par hasard un numéro de
     facture payée d'au moins 6 caractères ?
2. **L1, forme sans référence** : le texte du geste sûr (AC-b13) est-il juste au code — la transaction `pending`
   empêche-t-elle la clôture du compte (`bank_accounts.rs`, I1) et figure-t-elle au contrôle K2 (`closing_checks.rs`) ?
   l'annulation du paiement est-elle refusée si l'exercice est clos (`supplier_settlement_cancel_blocker`) ?
3. **`excluding` retiré** : plus aucune trace de ce paramètre dans un AC ou une mutation hors des mentions datées ;
   l'effet de bord écrit (vue périmée) est-il exact ?
4. **Contacts par tranches de 500** : `find_contacts_by_ids` (`repositories/reconciliation.rs:681-700`) refuse-t-il
   bien au-delà de 500 ? la carte de contacts d'`accept_one_rule` est-elle prescrite sans ambiguïté ?
5. **Résidus** : `grep -nF` sur la fiche (hors Change Log) et le registre — « ses deux codes » doit nommer les deux
   codes ; « les deux lignes qui » absent ; C-30-1-32, 35, 37, 38 annotés.
6. **Recomptes** : 26 AC, 24 tâches, 64 tests nommés (T-a1 à a17 sauf a9 ; T-b1 à b50 sauf b43, b44 ; 7 Vitest), 43
   lignes de mutation (40 à exécuter), 41 choix — recompte chacun.

Un défaut introduit par la remédiation est au moins **MEDIUM** ; un faux rattachement ou un double comptage muet créé
par la 30-1 est **HIGH**.

## Exigences de preuve

Toute affirmation de présence ou d'absence s'appuie sur une sortie `grep -nF` copiée ; tout fait du code, sur sa ligne.
⚠️ Un finding qui reproche à la fiche l'**absence** d'un code non encore écrit (la fiche prescrit, le développement
écrit) est une erreur de catégorie : vérifie qu'il s'agit bien d'un défaut de la **prescription**.

## Lecture seule stricte

⛔ N'écris aucun fichier hors de ton rapport. Aucune commande qui écrit, compile ou exécute : `scripts/*` (dont
**`scripts/prepare-release.sh`**), `make`, `latexmk`, `sed -i`, `perl -pi`, `git commit`/`add`/`checkout` **sous aucune
forme**/`switch`/`restore`/`stash`/`reset`/`rebase`/`tag`/`push`/`worktree`, `cargo`, `npm`, `npx`, `node`, **tout
interpréteur** (`python3`, `python`, `perl`, `ruby`, `bash`/`sh` sur un script), `docker`, **`mariadb`/`mysql`** et toute
requête SQL, `gh` en écriture. Aucune donnée réelle de Guy. Autorisés : lecture, `grep`, `sed -n`,
`git log`/`show`/`diff`, `gh api` en lecture.

## Rapport

Dans `/home/gcorbaz/devel/kesh-gate-logs/301-validate-p6-T.md`, **écrit d'un seul jet** par l'outil d'écriture de
fichier (aucune retouche par commande, même sur ton rapport — s'il faut le corriger, réécris-le en entier). Findings
numérotés `T1-…`, sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`, preuve, correction proposée. ⛔ **La
liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne compte pas. Dernier message : le chemin du
rapport, le bilan par sévérité, une ligne par finding MEDIUM ou plus.
