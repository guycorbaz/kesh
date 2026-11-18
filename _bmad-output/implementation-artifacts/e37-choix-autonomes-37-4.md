# Epic 37 — choix autonomes de la story 37-4 (rapports en devise)

*Registre par union (`docs/agents-orchestration.md` § 1). Développement du 2026-11-16, worktree `kesh-s37-4`, branche
`story/37-4-rapports-en-devise` partie de `946aa86a` (37-3a, modèle).*

## C-37-4-1 — Colonnes en devise du CSV, et la mention de réévaluation

- **Contexte** : AC-5 fixe la règle des colonnes (seulement si le rapport contient un compte en devise) sans en nommer
  les en-têtes ; AC-2 veut la mention « Réévaluation au cours X » en littéral français au CSV.
- **Choix** : grand livre `Devise;DebitDevise;CreditDevise;Cours;SoldeProgressifDevise;MentionDevise` ; balance
  `Devise;SoldeOuvertureDevise;TotalDebitDevise;TotalCreditDevise;SoldeClotureDevise` ; bilan et compte de résultat
  `Devise;SoldeDevise`. Les colonnes s'ajoutent **après** celles d'avant (un script qui lit par position garde ses
  colonnes). La mention de réévaluation va dans `MentionDevise`, les montants en devise d'une réévaluation restent
  vides (jamais un texte dans une colonne de montant). Lignes de total, ligne calculée du résultat reporté et comptes
  CHF : colonnes vides.
- **Écartés** : la mention dans `Libelle` (change le sens d'une colonne existante) ou dans `Cours` (texte dans une
  colonne numérique).
- **Réversible** : oui (format d'export, aucune donnée).

## C-37-4-2 — Montants en devise d'une ligne du grand livre

- **Choix** : `debitCurrency` et `creditCurrency` sont **tous deux** présents sur toute ligne d'une section en devise,
  0 du côté opposé — comme `debit`/`credit` en CHF ; une réévaluation porte 0 des deux côtés (JSON), et son affichage
  (écran, PDF, CSV) n'écrit aucun montant en devise (AC-2).
- **Écarté** : un montant unique sans côté (le côté se lirait alors sur le CHF, double lecture).
- **Réversible** : oui.

## C-37-4-3 — `balances_as_of` : forme du résultat

- **Choix** : `Vec<(i64, Decimal, Option<Decimal>)>`, type nommé `SoldeALaDate`, trié par numéro de compte puis id —
  la forme littérale de l'AC-1 quater ; `account_ids = Some(ids)` rend aussi un compte CHF demandé, avec `None` en
  devise.
- **Écarté** : une struct nommée (la fiche 37-5 cite la forme tuple).

## C-37-4-4 — Disposition du PDF

- **Choix** : grand livre — la devise au bandeau du compte (« 1020 — Banque USD · USD ») ; seconde ligne en petit
  (6,5 pt, 3 mm) sous chaque mouvement, à la colonne du libellé ; le solde en devise des lignes d'encadrement
  (ouverture, rupture, « À reporter », « Report », clôture) dans la colonne des contreparties, vide sur ces lignes ;
  les totaux en devise en seconde ligne sous le total des mouvements. Un mouvement en devise réserve sa seconde ligne
  avant le saut de page. Balance — seconde ligne sous le compte, réservée avec lui. Bilan et compte de résultat —
  « USD 1'233.57 » à 85 mm du bord droit, entre le nom et le solde.
- **Réversible** : oui.

## C-37-4-5 — Cours affiché

- **Choix** : `format_exchange_rate` (PDF, CSV) et, à l'écran, le `formatRate` existant de la 37-2
  (`exchange-rates-grid.ts`, même règle : zéros de queue retirés, 4 décimales au moins, 10 au plus) — DRY côté
  frontend ; un cours de 0.95 s'écrit donc `0.9500`.

## C-37-4-6 — Signe du solde en devise au bilan et au compte de résultat

- **Choix** : une fonction `balance_sheet::sign_exprs(type)` rend les deux expressions (CHF et devise) par **une**
  `match` (sur `is_debit_natured`), partagée par le bilan et le compte de résultat ; les deux `unreachable!` d'avant
  deviennent des `assert!` de même portée (un type de compte inattendu panique comme avant).

## C-37-4-7 — Fixtures de non-régression mono-devise

- **Choix** : enregistrées au commit de base (`88955a09`, code de `946aa86a`) avant toute modification — CSV, JSON et
  **lignes de texte du PDF** (opérateurs `Tj` du PDF non compressé du profil de test) des cinq rapports (journal
  compris, AC-7), pour une société CHF et une société multi-devises au compte USD archivé sans ligne. Le test
  `rapports_mono_devise` les compare octet pour octet ; elles ne se réenregistrent que sur décision
  (`KESH_FIXTURES_ENREGISTRER=1`).

## C-37-4-8 — Libellés en devise du PDF

- **Choix** : six champs neufs de `SectionLabels` (« Réévaluation au cours », « solde », « ouverture », « débit »,
  « crédit », « clôture »), défaut français ; le handler construit toujours `PdfContext::fr_ch_default`
  (`routes/reports.rs`), comme pour `ledger_fy_break` (AC-2). À l'écran, six clés i18n `reports-fx-*`, quatre
  locales.

## C-37-4-9 — Test « CHF seulement » de la balance âgée

- **Constat** : la balance âgée lit les factures, non les lignes d'écriture (D7) ; une écriture en USD ne peut pas
  l'atteindre. Le test vérifie qu'elle reste sans champ en devise et sans le montant de l'écriture USD — le seul
  contrôle possible à ce niveau.
