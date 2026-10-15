# Prompt — revue de code P2 **ciblée** de la Story 30-1b (une lentille — Haiku)

Tu relis **la seule remédiation de la revue de code P1** de la **Story 30-1b — proposer et accepter les factures
fournisseur pour un débit** de Kesh (comptabilité suisse, Rust/Axum + SvelteKit + MariaDB 10.11), dans le worktree
`/home/gcorbaz/devel/kesh-30-1`, branche `story/30-1b`. Modèle : **Haiku**. Contexte frais : ne présume rien du travail
de l'agent. Langue du rapport : français.

**Le diff à relire, aplati, et lui seul** :

```sh
git -C /home/gcorbaz/devel/kesh-30-1 diff a83ad2c4 HEAD
```

`a83ad2c4` est le dernier commit de la story **avant** la remédiation (le report sur `main` `f1508653` est **hors** de ce
diff : ne relis ni la 30-2a, ni le code de la 30-1b antérieur à la remédiation, sauf pour comprendre une ligne du diff).
Lis aussi, pour l'intention : les trois rapports de la P1 (`/home/gcorbaz/devel/kesh-gate-logs/301b-review-p1-{B,E,A}.md`),
le registre `_bmad-output/implementation-artifacts/epic-30-choix-autonomes-301.md` **C-30-1-60 à C-30-1-64**, et, dans la
fiche `_bmad-output/implementation-artifacts/30-1-reconciliation-factures-fournisseur.md`, l'encadré **L1** et les angles
morts **L10 à L12** (§ *Angles morts assumés*), ainsi que l'entrée *Revue de code P1 de la 30-1b* du Change Log.

## Interdits absolus

- **Lecture seule stricte.** N'écris, ne modifie, ne crée **aucun** fichier, hormis ton rapport — pas même un fichier
  temporaire (pas de redirection `>` vers un fichier, ni dans `/tmp`, ni dans un scratchpad).
- **Aucun `git checkout` ni `git switch`, sous aucune forme** (ni `--detach`, ni sur un fichier), aucun `git` qui écrit
  (`commit`, `stash`, `reset`, `rebase`, `tag`, `push`, `restore`).
- **N'exécute rien qui écrit, compile ou exécute** : ni `scripts/prepare-release.sh`, ni `scripts/test-fast.sh`, ni
  `scripts/mem-guard.sh`, ni `cargo`, ni `npm`/`npx`, ni `make`, ni `latexmk`, ni `python3`, ni `sed -i`, ni `perl`, ni
  `docker`, ni `mariadb`. Aucune commande `gh`.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `grep`, `sed -n`, `cat`, `ls`, `wc`, et
  `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` **vers la sortie standard seulement**, suivi
  d'un `grep -o` (jamais vers un fichier).
- Aucune donnée réelle dans le rapport.

## La lentille — chasseur de régressions de la remédiation

*La sévérité se déplace vers ce qu'on vient d'écrire* (`CLAUDE.md`, § *La passe ciblée*). Cherche ce que **ce diff**
casse ou laisse incohérent, axe par axe :

1. **`contact_for_score`** (`crates/kesh-api/src/routes/reconciliation.rs`) — les deux appels (acceptation client et
   fournisseur) : la garde `contact_id <= 0`, l'erreur rendue en `FailedProposal` `DATABASE_ERROR` (et non en `AppError`,
   § *Pattern batch* du `CLAUDE.md`), le `?` à chaque site : la fonction appelante rend-elle bien
   `Result<_, FailedProposal>` ? Un comportement client a-t-il changé ailleurs que pour une erreur de base ?
2. **Libellés de `RECONCILIATION_RULE_MASKED`** (quatre locales `crates/kesh-i18n/locales/*/messages.ftl` et le repli de
   `frontend/src/lib/features/reconciliation/failed-proposal-label.ts`) : pour chaque raison (`payment_batch`,
   `invoice_candidate`, `invoice_slightly_overpaid`, générique), le geste nommé est-il **juste** au code et au manuel ?
   Le repli égale-t-il **mot pour mot** la valeur fr-CH ? Les trois autres locales disent-elles la même chose (sens, pas
   de conseil inconditionnel d'« Affecter manuellement », aucun conseil de **confirmer** un lot) ? Cohérence avec
   l'encadré L1 du manuel (« Dépense déjà réglée par un autre chemin ») : un libellé conseille-t-il un geste que L1
   interdit ?
3. **Paiement antérieur à la facture** : `reconciliation-failed-payment-before-invoice` doit être **identique** à son
   texte de `fecc8566` dans les quatre locales (`git show fecc8566:crates/kesh-i18n/locales/fr-CH/messages.ftl | grep …`),
   et la clé neuve `…-days` servir toute tolérance > 1. Le repli `toleranceDays` (1 pour toute valeur invalide) mène-t-il
   à la bonne clé ? Le manuel cite-t-il encore exactement l'écran (« plus d'un jour ») ?
4. **Compteurs** : `sitesTotal` de `frontend/src/lib/shared/i18n-keys.test.ts` (2219) — recompte le `+1` par
   `grep -o "i18nMsg(" frontend/src/lib/features/reconciliation/failed-proposal-label.ts | wc -l` à `a83ad2c4` et à
   `HEAD`.
5. **Tests Vitest modifiés** (`failed-proposal-label.test.ts`) : une assertion qui passerait à vide, un cas `undefined`
   mal formé, un test qui ne distingue pas l'ancien texte du nouveau.
6. **E2E** : `frontend/tests/e2e/reconciliation-link.spec.ts` appelle `wiredBankAccountViaApi(page, TEST_IBAN)` à
   l'intérieur d'un bloc qui tient déjà un contexte API (`ctx`) : deux contextes ouverts à la fois — fuite, ordre de
   fermeture, ou effet sur le compte câblé ? Une importation inutilisée restée (`APIRequestContext`) ?
7. **Textes** : `docs/manual/fr/user-manual.tex` **et le PDF** (aplati, sortie standard) — exigibilité « la plus tardive
   de sa date et de son échéance » (comparer à `SUPPLIER_INVOICE_EXIGIBILITY_DATE_SQL`), le lot oublié (« sans limite de
   date » : vrai au code, `find_open_supplier_invoice_in_account_batch` ?), *Application des règles* ; `docs/api-external.md`
   (`amountDue` à quatre décimales — vrai pour la valeur sérialisée ? le chemin `…/transactions/{id}/link` existe-t-il
   sous ce nom ?) ; `CHANGELOG.md` § [0.14.0] (bandeau « À faire » et entrée *Modifié*). Le PDF est-il à jour du `.tex` ?
8. **Comptes rendus** : le Dev Agent Record et le Change Log de la remédiation (fiche) — chaque chiffre déclaré
   (tests, gates, mutations) se retrouve-t-il dans les journaux `/home/gcorbaz/devel/kesh-gate-logs/301b-p1rem-*` ?

## Rapport — d'un seul jet

Écris ton rapport **en une seule fois**, à la fin, dans `/home/gcorbaz/devel/kesh-gate-logs/301b-review-p2-T.md` :

- le décompte `CRITICAL · HIGH · MEDIUM · LOW` ;
- chaque finding : sévérité, `fichier:ligne`, preuve (**sortie `grep -nF` copiée** pour toute affirmation de présence ou
  d'absence), correctif proposé ;
- **la liste des axes réellement exercés (1 à 8) et de ceux qui ne l'ont pas été**, avec la raison. Un « 0 finding » sans
  cette liste ne compte pas comme passe.

⚠️ Un finding qui affirme qu'un patch « n'a pas été appliqué » se vérifie par `grep -nF` sur le fichier **à HEAD**, pas
sur une ligne du diff.
