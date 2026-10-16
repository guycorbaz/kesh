# Prompt — passe ciblée P2 de la revue de code, Story 30-2b (la création côte à côte)

*Versionné le 2026-10-16. **Passe ciblée** (`CLAUDE.md` § *La passe ciblée*) : **une seule lentille**, contexte frais,
**Haiku**, braquée sur le **seul diff de la remédiation de la revue P1**. Lecture seule stricte.*

Dépôt `/home/gcorbaz/devel/kesh-30-2`, branche `story/30-2b`. Le diff à relire, **aplati**, et lui seul :

```sh
git -C /home/gcorbaz/devel/kesh-30-2 diff 5da6428d HEAD -- . ':(exclude)_bmad-output/implementation-artifacts/30-2b-review-prompt-p2-ciblee.md' ':(exclude)*.pdf'
```

`5da6428d` = l'état revu en P1 ; `f428279f` = le **dernier commit de code** de la remédiation ; les commits suivants
ne portent que la documentation (manuels et PDF, `docs/api-external.md`), la fiche, le registre et
`sprint-status.yaml`. ⚠️ Pathspec `':(exclude)…'`, **non** `':!…'`.

Rapports qui l'ont motivée : `/home/gcorbaz/devel/kesh-gate-logs/302b-review-p1-B.md`, `302b-review-p1-E.md`,
`302b-review-p1-A.md`. La remédiation est décrite à la fiche
`_bmad-output/implementation-artifacts/30-2-documents-factures-fournisseur.md` (Dev Agent Record § *30-2b — remédiation
de la revue de code P1*, entrée datée « Revue de code P1 de la 30-2b » du Change Log, § *Angles morts assumés*) et au
registre `epic-30-choix-autonomes-302.md`, **C-30-2-41 à 44**. Règles : `CLAUDE.md` (§ *La passe ciblée*, § *Une passe
qui ne déclare pas ses axes exercés ne compte pas*, § *Haiku-specific guardrails*, § *Le prompt d'une passe doit
NOMMER le manuel*, § *Un appariement automatique propose, il ne crée jamais*) ; `docs/agents-orchestration.md` § 6.

⚠️ **Lis le diff aplati ci-dessus, pas une suite de commits.** Ce que la branche disait avant `5da6428d` n'est pas
l'objet de la passe.

## Lentille T — régression de la remédiation P1

Pour **chaque ligne ajoutée** du diff qui affirme un fait (comportement, texte, chemin, nombre), vérifie-le au code
par `grep -nF` ou `sed -n` et **copie la sortie**. Cherche aussi ce que le patch a pu casser autour de lui. Axes, tous
à exercer :

1. **La touche Entrée dans le panneau** (`frontend/src/lib/features/supplier-invoices/SupplierCreatePanel.svelte`,
   `onPanelKeydown`) : le panneau vit dans le `<form>` de `SupplierInvoiceForm.svelte`. Le `preventDefault()` sur le
   `keydown` (gestionnaire délégué de Svelte 5) annule-t-il bien la soumission implicite dans un navigateur ? Les
   cibles : seuls les `<input>` — un `<select>` du panneau ou un bouton peut-il encore soumettre la facture ? Un
   double Entrée pendant la création (`saving`) ? Une composition IME (`isComposing`) ? Entrée dans un champ **hors**
   panneau soumet-il toujours la facture (le comportement voulu ailleurs n'est-il pas cassé) ? Le
   `svelte-ignore a11y_no_noninteractive_element_interactions` est-il justifié, et le retrait du
   `svelte-ignore state_referenced_locally` devant `contactType` (A8) laisse-t-il `npm run check` sans avertissement
   neuf (lis `/home/gcorbaz/devel/kesh-gate-logs/302b-gate3-check.log`) ?
2. **Les tests de la touche Entrée** (`SupplierInvoiceForm.test.ts`, trois tests neufs et la fonction `pressEnter`) :
   jsdom n'implémente pas la soumission implicite ; le modèle (keydown annulable, puis `requestSubmit()` s'il n'a pas
   été annulé) est-il fidèle au navigateur ? Le test « chaque champ » parcourt-il **réellement** 6 champs puis 7
   (fournisseur sélectionné, lignes valides — sinon `submit()` s'arrêterait avant `onSubmit` et le test passerait à
   vide) ? Le **témoin** prouve-t-il que `pressEnter` soumet depuis un champ de la facture ? Le pas E2E
   (`frontend/tests/e2e/supplier-documents.spec.ts`, E2 : `press('Enter')` dans le nom) établit-il ce qu'il nomme
   (sans fournisseur sélectionné, une soumission aurait produit « Sélectionnez un fournisseur » et laissé le panneau
   ouvert) ?
3. **La référence libre** (`crates/kesh-api/src/routes/supplier_invoices.rs` : `is_payment_blank`,
   `normalize_payment_field`, `trim_matches`) : l'ensemble de blancs est-il **le même** qu'avant pour la normalisation
   (aucun caractère ajouté ou retiré) ? La longueur est-elle comptée après le rognage, en caractères ? Un U+200B ou
   U+FEFF **interne** reste-t-il (voulu) ? Les tests neufs de `crates/kesh-api/tests/inbox_import_e2e.rs` (64 et
   65 « é », U+FEFF + 64 + U+200B, référence blanche avec QR-IBAN → `QRR_REQUIRED`, contact d'une autre société
   écarté de `supplier-candidates`) prouvent-ils ce qu'ils nomment ? Pour le dernier : l'état posé en SQL
   (`UPDATE contacts SET company_id`) est-il atteignable par une FK composite ou un déclencheur qui le refuserait
   (`grep -rn "REFERENCES contacts" crates/kesh-db/migrations/`) ?
4. **`iban.ts` et `iban.test.ts`** : l'expression régulière construite par `new RegExp('[\\s' +
   String.fromCodePoint(0x85, 0x200b) + ']', 'g')` équivaut-elle à l'ancienne `/[\s\u0085​]/g` (drapeau `g`,
   `lastIndex` d'une expression globale partagée entre appels de `replace` — sans effet pour `replace`, à vérifier) ?
   Les sept points de code du test sont-ils ceux du test Rust (`create_accepts_and_normalizes_payment_details`) ?
   Reste-t-il un caractère invisible littéral dans ces deux fichiers (`grep -cP "[\x{00A0}\x{202F}\x{200B}\x{FEFF}\x{0085}]"`) ?
5. **Doc-comments, registre, angles morts** : le doc-comment de `complete_import` (C-30-2-42) et celui de
   `validate_payment_details` (montant attendu) disent-ils vrai ? Les justifications de C-30-2-42 (« patron de
   `pay`… ») et C-30-2-43 se vérifient-elles au code (`grep -n "load_with_settlement_cancellation"
   crates/kesh-api/src/routes/*.rs` ; `frontend/src/routes/(app)/supplier-invoices/new/+page.svelte`) ?
6. **Manuels et PDF** : `docs/manual/fr/user-manual.tex` (le panneau et la touche Entrée ; la note du montant attendu ;
   la phrase des réglages de facturation sans « complétion ») et `admin-manual.tex` (deux phrases du même objet),
   `docs/api-external.md` (référence rognée, montant attendu) : chaque affirmation est-elle vraie au code ? Le **PDF**
   dit-il la même chose que le `.tex` — contrôle **aplati vers la sortie standard seulement** :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oF "touche Entrée"` (idem pour
   « montant attendu saisi à la main » et, au manuel d'administration, « créées depuis un document importé ») ; et
   « (ou la complétion) » a-t-il disparu des deux PDF ?
7. **Fiche et comptes rendus** : la remédiation a-t-elle **réécrit** une entrée passée (Change Log, registre, Record)
   au lieu de l'**annoter** (interdit) ? Les décomptes écrits (tests ajoutés, mutations M-b20 à M-b25, gates) se
   retrouvent-ils dans le dépôt et dans les journaux cités (`/home/gcorbaz/devel/kesh-gate-logs/302b-mutations-p1.log`,
   `302b-mut-p1-*.log`, `302b-gate3-*.log`, `302b-e2e-p1.log`) ? Recompte :
   `grep -cE "^\s*it\(" frontend/src/lib/features/supplier-invoices/SupplierInvoiceForm.test.ts` aux deux bornes
   (`git show 5da6428d:<f>` et le fichier final).
8. **Mots-clés de fermeture** : `git -C /home/gcorbaz/devel/kesh-30-2 log 5da6428d..HEAD --format=%B | grep -niE "\b(close[sd]?|fix(e[sd])?|resolve[sd]?):?\s+#[0-9]+"` doit être vide.

**Haiku — garde-fous** (`CLAUDE.md`) : tout finding qui affirme l'**absence** d'un patch ou la **présence** d'un
anti-pattern s'appuie sur une sortie `grep -nF` copiée, sur le **fichier final** et non sur une ligne d'un hunk ; ce que
la fiche **prescrit** à la sous-story 30-2c (document joint, `sourceDocument`, détacher) n'existe pas encore — son
absence n'est **pas** un défaut ; ne signale que ce que le diff **affirme** et qui est faux, ou ce qu'il casse.

## Ce que tu rends

Findings numérotés `T1…` avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, fichier et ligne, **preuve** (commande
exécutée et sortie copiée), correction proposée. ⛔ **La liste des axes exercés ET non exercés** (1 à 8 ci-dessus) — un
« 0 finding » sans elle ne compte pas comme passe. Rapport **écrit d'un seul jet**, un seul fichier, à la fin, sans
brouillon ni fichier intermédiaire : `/home/gcorbaz/devel/kesh-gate-logs/302b-review-p2-T.md`. Dernier message : le
chemin du rapport, le bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport — ni dans `/tmp`, ni ailleurs, pas même temporairement. Lecture seule
stricte : **aucun `git checkout`, `git switch`, `git checkout --detach` ni aucune autre forme** ; aucune commande qui
écrit, compile ou exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`,
`scripts/mem-guard.sh`, `scripts/regen-test-schema.sh`), `make`, `latexmk`, `xelatex`,
`git commit`/`add`/`stash`/`fetch`/`pull`/`reset`/`rebase`/`tag`/`push`, `cargo`, `npm`, `npx`, `node`, **`python3`,
`python`, `perl`, `ruby` et tout autre interpréteur**, `sed -i`, `docker`, **`mariadb`, `mysql` et toute requête SQL**,
`bash` sur un script du dépôt, `gh` en écriture. `pdftotext` vers la **sortie standard** seulement, jamais vers un
fichier ; aucune redirection `>` vers un fichier. Aucune donnée réelle de Guy ne se lit ni ne se cite (rien de
`~/kesh-test/`). Autorisés : lecture, `grep`, `sed -n`, `awk` en lecture, `git log`/`show`/`diff`/`blame`,
`pdftotext … -`.
