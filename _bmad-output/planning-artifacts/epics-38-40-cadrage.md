# Epics 38 à 40 — cadrage (PayPal, Wise, Stripe)

**Demande du responsable de l'epic (contributeur), 2026-11-14/15.** Trois epics d'import automatique des comptes de prestataires de
paiement, après l'Epic 37 (multi-devises), dont ils dépendent. Numéros : 35 et 36 réservés ailleurs ; **38 PayPal**,
**39 Wise**, **40 Stripe (numéro à confirmer par Guy)**. Une branche par story, fusionnée localement dans la branche de
son epic, puis **une PR par epic** vers le dépôt de Guy, ouverte depuis le dépôt du contributeur, après celle de l'Epic 37. Le développement
attend celui de l'Epic 37, lui-même en attente du retour de Guy sur la page de revue.

## Demande

- **38 PayPal** : (1) télécharger automatiquement les transactions PayPal, frais, conversions de devises et paiements
  faits avec PayPal compris ; (2) les imputer aux bons comptes — sorties aux comptes de charges, entrées aux comptes de
  produits selon la boutique — par les **Règles d'affectation** existantes, étendues (les factures sont le plus
  souvent hors de Kesh) ; (3) relevé annuel : le *custom account statement* PDF de PayPal.
- **39 Wise** : (1) télécharger automatiquement les transactions Wise : frais, conversions, paiements envoyés et reçus,
  achats par carte ; (2) générer et télécharger les relevés de compte.
- **40 Stripe** : les mêmes éléments pour Stripe, avec un compte intermédiaire (délai de versement, frais,
  conversions).

## Décisions du responsable de l'epic (questions/réponses du 2026-11-14/15)

- **B1 — Comptes** : PayPal en compte **business** (application REST *live*, permission *Transaction Search*). Deux
  comptes PayPal (un par usage) et un compte Wise, chacun avec trois soldes (CHF, EUR, USD) : **un compte
  Kesh par solde et par devise** (9 comptes), comptes en devise de l'Epic 37. Exemple : plan 2019 du responsable de l'epic
  (`data-not-committed/2024/`, hors dépôt ; ébauche de correspondance `mapping-comptes-2024.csv` : PayPal 1030–1033,
  Wise 1034–1036, Débiteurs Stripe 1101–1103, frais d'encaissement 6910–6914, frais bancaires 6901).
- **B2 — Nouveau solde** : un solde de devise sans compte Kesh est **listé et proposé** ; l'Administrateur confirme la
  création. Jamais de création d'office (`CLAUDE.md`, *Un appariement automatique propose, il ne crée jamais*).
- **B3 — Frais** : un compte **de regroupement** des frais, et **un sous-compte par prestataire et par devise**, pour
  rapprocher chaque sous-compte du relevé mensuel du prestataire. Chaque transaction se passe **au brut** par la règle ;
  ses frais vont au sous-compte de frais du prestataire et de la devise.
- **B4 — Conversions** internes au prestataire : conversion de l'Epic 37 par le compte de transit,
  `completeDifference: true` (A31) ; coût de conversion dans la ligne de différence, ou au compte de frais quand le
  prestataire le donne à part.
- **B5 — Règles d'affectation** : nouveaux critères — courriel du payeur ou du marchand, préfixe de facture, nom
  d'article, code d'événement du prestataire, sens (entrée/sortie). Une règle **propose** ; l'utilisateur accepte en
  lot (`accept_batch`, *Pattern batch*). Aucune écriture d'office.
- **B6 — TVA** : la société du responsable de l'epic est **assujettie** ; la TVA est calculée par ses boutiques, non par PayPal ou
  Stripe. Une écriture proposée **sépare la TVA** selon la **règle TVA par défaut du compte** de produit ou de charge
  (code et taux), nécessaire au décompte — p. ex. l'**impôt sur les acquisitions** pour un achat de service à
  l'étranger sans TVA. Kesh n'a aujourd'hui **aucun code TVA par défaut sur un compte** : à créer. ⚠️ Les dix P1 de
  la TVA (jalon « Vague 2 ») restent ouverts : le responsable de l'epic ne dépose pas de décompte TVA par Kesh cette année et passe la TVA
  par l'API et ces imports seulement. À dire à Guy dans les epics (le report des P1 TVA reposait sur un seul
  utilisateur non assujetti).
- **B7 — Remboursements et litiges** : un remboursement est rapproché de sa transaction d'origine et proposé comme
  l'inverse de son écriture (mêmes comptes, TVA comprise) ; un litige ou une rétrofacturation va dans une file « à
  examiner », sans proposition ; seuls les mouvements qui changent le solde se comptabilisent (blocages et réserves
  sans mouvement : non).
- **B8 — Date de départ** : chaque compte de prestataire a une date « télécharger depuis » (ici 1.1.2024) ; rien
  d'antérieur. PayPal ne remonte que trois ans.
- **B9 — Téléchargement** : **réglage** par compte de prestataire : quotidien automatique, ou à la demande seulement
  (bouton « télécharger maintenant » dans les deux cas).
- **B10 — Relevés** : **PayPal** : aucune API ; le responsable de l'epic télécharge le *custom account statement* annuel et le **joint à
  la main** (document de l'exercice et du compte). **Wise** : PDF par l'API (`balance-statements/…/statement.pdf`),
  avec l'authentification forte (SCA : défi signé par une clé privée dont la clé publique est déposée chez Wise,
  environ tous les 90 jours) ; rangé comme document du compte et de l'exercice. **Stripe** : pas de PDF ; rapports
  joints à la main comme PayPal.
- **B11 — Secrets** : enregistrés par société en base, **chiffrés** par une clé tirée d'une variable d'environnement
  neuve ; saisis par un Administrateur (écran ou session API), jamais réaffichés.
- **B12 — Tests** : aucun appel réseau en CI ; réponses PayPal, Wise, Stripe enregistrées et anonymisées ; recette
  sur les comptes réels du responsable de l'epic sur une instance jetable, avec des accès en lecture seule.
- **B13 — Stripe (40)** : une vente Stripe se passe au **compte intermédiaire** de la devise (Débiteurs Stripe :
  produit net + TVA séparée, B6) ; frais et conversions sortent de ce compte ; le versement est un virement de ce compte
  vers la banque (UBS), rattaché à la ligne du relevé bancaire comme un virement entre ses propres comptes (Epic 34) ; à
  la clôture, le solde est une créance. **PayPal** : la vente se passe directement au compte PayPal de la devise
  (versement instantané).
- **B14 — Socle commun (DRY)** : les premières stories de l'Epic 38 posent ce que partagent les trois prestataires :
  stockage chiffré des secrets, téléchargement planifié ou à la demande, type de compte bancaire « prestataire » lié à
  un compte du plan **déjà en devise** (engagement D8 de l'Epic 37, numéro, historique et soldes gardés), critères neufs
  des règles, code TVA par défaut d'un compte et séparation de la TVA dans les propositions. Les Epics 39 et 40
  n'ajoutent que leur prestataire.
- **B15 — Hors périmètre** : factures (création ou import) ; décompte TVA (P1 « Vague 2 »).

## Sources consultées (2026-11-14)

- PayPal Transaction Search : <https://developer.paypal.com/api/transaction-search/v1/search-get> — 31 jours par
  requête, trois ans d'historique, jusqu'à 3 h de délai, 500 par page, 10 000 par requête.
- PayPal relevés : <https://developer.paypal.com/reports/monthly-custom-statements> — demandés et téléchargés dans
  l'interface web ; aucune API de relevé PDF trouvée.
- Wise relevés : <https://docs.wise.com/api-reference/balance-statement>, jetons personnels
  <https://docs.wise.com/api-docs/features/authentication-access/personal-tokens>, SCA
  <https://docs.wise.com/guides/developer/auth-and-security/sca-over-api>.

## Décisions du responsable de l'epic — suite (2026-11-15, sur le projet d'epic 38)

- **B16 — Aucune migration breaking.** Le caractère « prestataire » d'un compte bancaire, ses comptes liés (frais et
  autres) et sa devise vivent dans les **réglages de la connexion du prestataire** (table neuve), non dans des valeurs
  neuves de colonnes existantes. Le compte reste `account_kind = 'bank'` (IBAN facultatif depuis 34-11a1) ; les
  critères neufs des règles vont dans des colonnes neuves ou une table enfant, non dans `match_type` ; les passages de
  téléchargement ont leur propre table, non un `source_format` neuf. Toutes les migrations ajoutent des tables ou des
  colonnes `NULL` : `kesh_version_min_required` inchangé. Un binaire antérieur voit un compte prestataire comme un
  compte bancaire ordinaire.
- **B17 — Exercice 2024** : déjà importé par l'API (recette de l'Epic 37) ; les transactions téléchargées de 2024 se
  **rattachent en lot** aux écritures existantes (signal « déjà comptabilisée » de 34-4), sans doublon.
- **B18 — Impôt sur les acquisitions** : compte distinct de TVA due sur acquisitions (p. ex. 2201).
- **B19 — Ventes** : sous-comptes de ventes par devise, regroupés en CHF, ventilés par catégorie de vente (la
  comptabilité analytique pourrait les remplacer : non tranché). Les règles d'affectation doivent proposer le bon
  compte de ventes **par domaine de boutique** : critère « boutique / domaine » explicite.
- **B20 — Remboursements et frais** : passés tels que le prestataire les rapporte. Les relevés sont les **sommes de
  contrôle** : solde du prestataire comparé au relevé.
- **B21 — WHMCS et CB Paid Subscriptions** : leurs exports CSV s'importent par un **script hors de Kesh**, par l'API ;
  l'API doit permettre à ce script de créer les écritures ou de les rattacher aux transactions de prestataire.

## Décisions du responsable de l'epic — suite (2026-11-15, après la révision B16–B21)

- **B22 — Une migration breaking acceptée** (remplace B16 sur ce point) : valeurs honnêtes `source_format =
  'PROVIDER_API'` et types de critère neufs des règles (`match_type`), sans détour (`MANUAL`, clé inerte
  `reference_contains`). `kesh_version_min_required` relevé (P2, P2-bis ; numéro de release à Guy). Les comptes liés
  et la devise restent dans les **réglages de la connexion** (B16 maintenu sur ce point).
- **B23 — Réglages d'un compte prestataire** : Kesh n'a pas de magasin générique de paramètres ; il a des colonnes
  `JSON` typées (`bank_profiles.column_mapping` → struct `ColumnMapping` ; `mcp_server_settings`). Les options propres
  au prestataire vont dans une colonne `settings JSON` lue en struct typée par prestataire (MariaDB : `JSON` = texte
  validé, pas de JSONB) ; **les références à des comptes** (frais par devise, autres comptes liés) restent des
  colonnes à clé étrangère (`ON DELETE RESTRICT`, gardes d'archivage) ; les secrets restent dans leur table chiffrée.
- **B24 — Q10** : chez PayPal, la boutique se reconnaît au **préfixe du numéro de facture** (`invoice_id`).
- **B25 — Q3** reportée ; **Q9** : Stripe = Epic **40** en attendant Guy.
- **B26 — Exigences** : chaque fiche des Epics 37 à 40 liste ses **FR** (exigences fonctionnelles) et **NFR**
  (non fonctionnelles), numérotées ; Guy dira s'il les veut en issues GitHub ou dans les fiches.

## Décisions du responsable de l'epic — suite (2026-11-15, questions de la rédaction)

- **B27 — Frais sur une conversion** : un frais donné à part sur la moitié créditée d'une conversion va au
  **sous-compte de frais** du prestataire et de la devise (B3), jamais dans la différence de change. Demande un
  amendement de 37-3b (A33, Epic 37) : l'écriture complétée d'une conversion admet en plus des lignes sur des comptes de
  charge de la même devise.
- **B28 — Frais 2024** : dans les livres 2024, les frais PayPal sont des **écritures à part**. Le rattachement en lot
  (38-6c) doit donc rattacher une transaction (montant net) à **plusieurs** écritures existantes — la vente au brut et
  l'écriture de frais — dont les mouvements sur le compte prestataire somment au net.
