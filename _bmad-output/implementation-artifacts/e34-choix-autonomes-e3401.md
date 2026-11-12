# Epic 34 — choix autonomes du développement de la 34-1 (clé `e3401`)

Story 34-1 (#966), l'assistant de démarrage pose la description et le compte comptable du compte bancaire. Choix tranchés au
développement (Opus 5.5, 2026-11-10), en complément de ceux de la fiche (`e34-choix-autonomes-e341.md`, C-e341-1 à 5, 33, 34).

**C-e3401-1 — le formulaire *Lier* de la page *Comptes bancaires* passe lui aussi par `linkableLedgerAccounts`, sans filtre sur le
numéro.** Contexte : la fiche (T4, C-e341-5) veut le filtre écrit une fois, pour « les deux écrans ». Le formulaire de lien
(`BankAccountJournalLinkForm.svelte`) en portait une troisième copie, avec un critère de plus : le numéro commence par 1 ou 2. Ce
critère contredit la 14-3a (aucun sens ne se déduit d'un numéro), le serveur ne le contrôle pas, et le formulaire de création de la
même page ne l'appliquait pas : un compte d'actif renuméroté hors des classes 1 et 2 se liait à la création, pas au lien. Retenu :
la fonction commune, le test du formulaire adapté (le compte `5000` de type actif y est désormais proposé), une ligne au CHANGELOG.
Autres : garder la copie (DRY non tenu, trois filtres divergents) ; ajouter une option `numberPrefix` (perpétue un critère faux).
Réversible : oui, une ligne.

**C-e3401-2 — liste des comptes en échec : le motif s'affiche sous le champ, sans toast.** Contexte : l'AC-6 veut « Aucun » et une
étape utilisable ; la garde `onboarding-catch-guard.test.ts` exige que tout `catch` de la page passe par `errorMessageOf`. Retenu :
`errorMessageOf(err, msg('onboarding-bank-ledger-load-error', …))` affiché sous le sélecteur (clé neuve, quatre locales), le champ à
« Non configuré », l'envoi possible. Autres : `catch` muet (rouge à la garde, et l'utilisateur ne saurait pas pourquoi la liste est
vide) ; toast (bruyant pour une étape qui reste utilisable) ; `.catch()` hors de la portée de la garde (contournement). Réversible :
oui.

**C-e3401-3 — l'option « Aucun » de l'AC-6 est l'option « Non configuré » de la page *Comptes bancaires*** (clé
`bank-accounts-labels-not-configured`, réutilisée comme le demande la T5). Un libellé neuf ferait dire deux mots différents à deux
écrans pour la même absence. Réversible : oui.

**C-e3401-4 — la lecture par rôle est `accounts::find_singleton_role_in_tx(tx, company_id, role, RoleLookupLock)`**, une énumération
à deux valeurs (`None`, `ForUpdate`) plutôt qu'un booléen, dans `repositories/accounts.rs`. Les trois lectures de
`insert_with_defaults_in_tx` la réemploient avec `ForUpdate` (même texte SQL qu'avant, au caractère près). Réversible : oui, interne.

**C-e3401-5 — le contrôle du compte lié, sur le pool, prend la société du jeton** (`current_user.company_id`), comme
`create_bank_account` ; le refus des rôles, dans la transaction, prend la société verrouillée (`lock_company`). Elles sont la même
société de l'installation à une société. Réversible : oui.

**C-e3401-6 — la présélection vit dans `format.ts`** : `PRESELECTED_BANK_LEDGER_NUMBER = '1020'` et `preselectedBankLedgerId(liste
filtrée)`, documentés comme un geste d'écran. Le serveur n'en sait rien (C-e341-4). Réversible : oui.

**C-e3401-7 — le refus des comptes de rôle s'applique à tout lien demandé (`journalAccountId` à une valeur), même identique au lien
déjà posé.** Un rejeu qui redonne un lien vers un compte devenu `Receivable` entre-temps est refusé, l'étape restant à 6 : la
finalisation, qui suit, aurait produit l'état que la 15-6c interdit. Autre : ne contrôler qu'un lien qui change (patron du `PATCH`) —
laisserait passer ce cas. Réversible : oui.
