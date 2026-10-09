# Guide de démarrage — Kesh comptabilité

Ce guide accompagne le premier usage de Kesh pour un indépendant suisse, une PME ou une association. Il complète le manuel administrateur (`docs/manual/fr/admin-manual.tex`) qui se concentre sur l'installation/configuration technique.

## 1. Vue d'ensemble

Kesh est un logiciel de comptabilité en partie double pour la Suisse, conforme :

- **QR Bill 2.2** — génération de factures avec QR code SIX.
- **pain.001.001.09** — fichiers de paiement ISO 20022 (Swiss Payment Standards / SIX).
- **CAMT.053.001.04** + CSV multi-encodage — import des relevés bancaires.
- **CO Art. 957–958f** — journal d'audit comptable, conservé 10 ans.

L'application est multilingue **FR / DE / IT / EN**, hébergée localement (sur le NAS Synology de l'utilisateur, par exemple). Les données ne quittent jamais la machine.

## 2. Premier accès

Au premier démarrage de Kesh (v0.1.2+), l'écran `/setup` affiche un formulaire de création du compte administrateur. Choisissez un nom d'utilisateur et un mot de passe robuste (≥ 12 caractères). Une fois validé, vous êtes connecté automatiquement.

Si vous oubliez votre mot de passe, le manuel administrateur explique la procédure de recovery break-glass via la variable d'environnement `KESH_ADMIN_PASSWORD`.

## 3. Configurer votre organisation (onboarding)

Après la connexion, l'onboarding vous guide, dans cet ordre :

1. **Langue de l'interface** (français, allemand, italien, anglais).
2. **Mode d'utilisation** — *Guidé* ou *Expert*.
3. **Chemin** — explorer Kesh avec des données de démonstration, ou le configurer pour la production. Les étapes suivantes sont celles du chemin production.
4. **Type d'organisation** (indépendant, association, PME) — il détermine le plan comptable que Kesh met en place.
5. **Langue comptable** — langue des libellés du plan comptable, indépendante de celle de l'interface. C'est à cette étape que Kesh met en place le plan comptable de votre type d'organisation.
6. **Coordonnées de l'organisation** (raison sociale ou prénom et nom, adresse, numéro IDE facultatif).
7. **Compte bancaire principal** — nom de la banque et IBAN, QR-IBAN facultatif ; l'étape peut être passée (« Configurer plus tard »).

À la finalisation, qui suit la dernière étape, Kesh crée votre premier exercice comptable : l'année civile en cours (du 1er janvier au 31 décembre). Aucune étape ne le demande ; vous le renommez ou en créez d'autres ensuite depuis **Administration → Exercices comptables**.

À tout moment, vous pouvez quitter l'onboarding et y revenir.

## 3 bis. Lier ses comptes bancaires au plan comptable

**Pourquoi ?**

Pour que la **réconciliation automatique** (FR47) puisse créer les écritures vers le bon compte comptable, et pour que la **page d'accueil affiche les soldes courants** de vos comptes, chaque `bank_account` doit être lié à un compte du plan comptable (classe 1 typique : Actif).

**Comment ?**

1. Naviguer vers **Administration → Comptes bancaires** (depuis la sidebar).
2. Cliquer sur le bouton « Lier » (ou « Modifier » pour l'édition complète) à droite du compte concerné.
3. Choisir le compte comptable dans le menu déroulant. Typiquement :
   - `1000 Caisse` pour la petite caisse.
   - `1020 Banque` pour un compte courant.
4. Valider.

**Cas multi-comptes courants** (ex. BCV + UBS) :

Si vous avez plusieurs comptes courants distincts, **NE PAS** lier les deux au compte parent `1020 Banque`. Sinon le solde affiché en page d'accueil agrégerait les deux et serait incorrect (la hiérarchie parent/enfants n'est pas remontée v0.1).

À la place :

1. Créer des **sous-comptes auxiliaires** via Administration → Plan comptable :
   - `1020.001 BCV CHF` (parent : `1020`).
   - `1020.002 UBS CHF` (parent : `1020`).
2. Lier chaque `bank_account` à son sous-compte respectif.

Le solde affiché en page d'accueil sera alors correct pour chaque compte bancaire séparément.

**Note v0.1** — le solde calculé inclut toutes les écritures de tous les exercices (« solde depuis création »). La fonctionnalité « solde de l'exercice courant » est planifiée pour Epic 12+.

## 4. Saisir des écritures

Menu **Mensuel → Écritures**. Saisie en partie double avec validation balanced.

## 5. Importer des relevés bancaires

Menu **Quotidien → Importer**. Glisser-déposer un fichier CAMT.053 ou CSV. Profils CSV réutilisables : **Administration → Profils bancaires**.

## 6. Réconcilier les transactions importées

Menu **Mensuel → Réconciliation**. Trois modes :

- **Automatique** avec score de similarité.
- **Manuelle** par sélection.
- **Éclatement** d'une transaction agrégée en plusieurs écritures.

Pour automatiser les écritures récurrentes (loyer, salaires) : créer des **règles d'affectation** via **Administration → Règles d'affectation**.

## 7. Facturer

Menu **Quotidien → Facturer**. Génération PDF avec QR Bill 2.2 conforme SIX.

## 8. Rapports

Menu **Mensuel → Rapports** : balance, résultat, bilan.

## 9. Export

Menu **Administration → Export global** : ZIP complet de toutes les données comptables (CO Art. 957 — souveraineté sur 10 ans).
