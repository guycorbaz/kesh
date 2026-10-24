# Epic 31 — registre des choix autonomes de la Story 31-5 (quatre défauts courts)

Versé par union dans `epic-31-choix-autonomes.md` au merge.

## Story 31-5

### C-31-5-0 — Emplacement de ce registre

- **Contexte** : le prompt de l'orchestrateur nomme `_bmad-output/planning-artifacts/epic-31-choix-autonomes-315.md` ;
  les consignes communes (`docs/agents-orchestration.md` § 1) et tous les registres de l'epic (`-311`, `-312`, `-313`)
  sont dans `_bmad-output/implementation-artifacts/`.
- **Retenu** : `implementation-artifacts/`, à côté des autres, pour que l'union se fasse au même endroit.
- **Écartée** : `planning-artifacts/` — un registre isolé que l'union ne verrait pas.
- **Réversible** : oui (un `git mv`).

### C-31-5-1 — Une seule story, quatre volets indépendants

- **Contexte** : cinq modules de premier niveau (`kesh-db`, `kesh-seed`, `kesh-api`, `kesh-i18n`, `frontend`) ; la règle
  de découpage préventif découpe **au-delà** de cinq.
- **Retenu** : une fiche, quatre volets A-D sans fichier de code commun (seuls les catalogues `kesh-i18n` sont partagés
  par C et D, en lignes distinctes).
- **Écartées** : quatre stories (quatre boucles de validation et de revue pour des correctifs d'une à six lignes de
  production) ; deux stories frontend/backend (découpage sans rapport avec les défauts).
- **Réversible** : oui — un volet qui résiste en revue se détache en story propre sans toucher les autres.

### C-31-5-2 — #687 et #645 écartées

- **Contexte** : les deux sont réelles. #687 : l'issue ne propose « aucune [piste] mécanique » (arrondi à la création
  qui change HT/TVA et la complétion d'une pièce importée, ou refonte de `pay_in_tx`). #645 : définir les jetons change
  l'apparence de tout l'écran et l'accessibilité, l'issue demande une story propre.
- **Retenu** : écartées, restent ouvertes ; #687 est la plus utile à Guy des deux (factures fournisseur réelles) et
  mérite une story à elle.
- **Écartée** : forcer quatre parmi les six en y mettant #687 à la place de #504 ou #469 — ce serait prendre la moins
  courte et la plus risquée pour tenir un chiffre.
- **Réversible** : oui.

### C-31-5-3 — #707 : corriger la démonstration, pas le chemin production

- **Contexte** : `insert_with_defaults_in_tx` sert la démonstration **et** la finalisation production ; aucun des deux ne
  désigne les comptes de TVA (voulu depuis la 18-1a). Pour un non-assujetti (Guy), le refus `CONFIGURATION_REQUIRED`
  d'une facture saisie par erreur avec un taux est un garde-fou.
- **Retenu** : une fonction distincte appelée par `seed_demo` seulement ; le chemin production garde `NULL`, et le manuel
  le dit (AC3) ; un test fige ce comportement production.
- **Écartées** : désigner les comptes dans `insert_with_defaults_in_tx` (change le comportement de toute installation
  neuve, retire le garde-fou du non-assujetti) ; manuel seul pour la démonstration (l'issue admet cette voie, mais la
  démonstration promet un essai sans réglage, et le correctif est court).
- **Réversible** : oui.

### C-31-5-4 — #707 : n'agir que sur une ligne de réglages née dans la transaction

- **Contexte** : `insert_with_defaults_in_tx` rend `inserted` ; si la ligne existait (`false`), elle a pu être lue et
  modifiée par quelqu'un, et une mise à jour sans `version + 1` casserait le verrou optimiste.
- **Retenu** : appel conditionné à `invoice_settings_created` ; `COALESCE` en plus (aucun écrasement). Mutation M4
  probablement équivalente, à déclarer.
- **Écartée** : mettre à jour toujours, avec `version + 1` — gère un cas que la démonstration (étape 2 → 3) ne produit
  pas.
- **Réversible** : oui.

### C-31-5-5 — #707 : l'entrée d'audit `installation.demo_seeded` reste telle quelle

- **Contexte** : elle porte `invoice_settings_created`, pas le détail des comptes.
- **Retenu** : inchangée (périmètre minimal).
- **Écartée** : y ajouter les trois comptes — utile mais hors du défaut.
- **Réversible** : oui.

### C-31-5-6 — #589 : la capture montre l'écran, la valeur de l'infobulle se lit au DOM

- **Contexte** : la recette visuelle exige une capture de tout contrôle visible touché ; une infobulle native `title`
  n'apparaît pas dans une capture Playwright.
- **Retenu** : capture de l'écran à trois exercices clos (boutons désactivés/actif) + valeur de `title` relevée au DOM et
  recopiée au Dev Agent Record ; Vitest l'assure.
- **Écartée** : remplacer l'infobulle native par un composant d'infobulle pour pouvoir la capturer — change le contrôle.
- **Réversible** : oui.

### C-31-5-7 — #504 : la traduction ne touche pas le rapprochement

- **Contexte** : l'issue demande de vérifier qu'aucun rapprochement ne s'appuie sur « Facture ».
- **Retenu** : établi au code — `reference_score` (`kesh-reconciliation/src/matching.rs:231-256`) compare le numéro de
  facture normalisé par contenance ; le préfixe n'y entre pas. Aucun test de rapprochement à ajouter.
- **Écartée** : garder le français « pour la banque » — la communication est lue par le débiteur, dans sa langue.
- **Réversible** : oui.

### C-31-5-8 — #504 : une clé dédiée, sans variable

- **Contexte** : réutiliser `invoice-pdf-title` (même valeur aujourd'hui) couplerait le texte lu par la banque au titre
  imprimé ; une clé `Facture { $number }` ferait entrer les marques d'isolation Fluent (U+2068/U+2069) dans la QR,
  aucun `set_use_isolating(false)` n'étant posé.
- **Retenu** : `invoice-qr-message-prefix` sans placeable, `format!("{prefix} {n}")` côté Rust, dans `kesh-api` (pas dans
  `I18N_KEYS` de `kesh-qrbill`, qui n'est pas touché).
- **Écartées** : `invoice-pdf-title` ; clé à variable ; clé ajoutée à `kesh-qrbill` (un sixième module pour rien).
- **Réversible** : oui.

### C-31-5-9 — #504 : pas de refigeage des PDF émis

- **Contexte** : depuis la 25-6-b, le PDF d'une facture est figé au premier rendu ; refigeage manuel seulement.
- **Retenu** : les PDF figés gardent leur communication française ; manuel le dit (AC10).
- **Écartée** : refigeage d'office (change une pièce émise, ce que la 25-6-b interdit).
- **Réversible** : sans objet.

### C-31-5-10 — #469 : le détail `chrono` d'une date mal formée n'est plus montré

- **Contexte** : `parse_date` renvoie aujourd'hui `{nom} invalide ({e})`, `e` étant le message anglais de `chrono`.
- **Retenu** : message traduit qui nomme le paramètre et le format attendu (`AAAA-MM-JJ`), sans `e`.
- **Écartée** : garder `e` en variable — un fragment anglais dans un message allemand ou italien.
- **Réversible** : oui.

### C-31-5-11 — #469 : `t_args` passe en `pub(crate)`

- **Contexte** : seule `t` est exposée ; `t_args` retire déjà les marques d'isolation.
- **Retenu** : élargir sa visibilité (DRY : pas de seconde fonction qui ferait la même chose dans `audit_log.rs`).
- **Écartée** : `format!` sur le rendu de `t` — perd l'ordre des mots propre à chaque langue.
- **Réversible** : oui.

### C-31-5-12 — Classe voisine non traitée : les autres `AppError::Validation` en français

- **Contexte** : des dizaines de routes rendent des `AppError::Validation` écrits en français ; #469 ne couvre que le
  journal d'audit et aucune issue ouverte ne couvre le reste.
- **Retenu** : hors périmètre ; issue **demandée** à l'orchestrateur (« on trace toujours »), même si elle fait remonter
  le compte.
- **Écartée** : les traduire ici — ni mécanique ni court (289 lignes au relevé grossier du 2026-10-24).
- **Réversible** : sans objet.
