# Epic 27 — choix autonomes de la Story 27-3 (à fusionner dans `epic-27-choix-autonomes.md`)

*Écrit le 2026-10-12 par l'agent de spécification (Opus 5.5), en autonomie. Le registre de l'epic vit sur la branche
de la 27-1 (`story/27-1-supprimer-import`, non mergée) : l'orchestrateur fusionne ce fichier par **union**, sans
renuméroter (préfixe `C-27-3-`, propre à cette story). Chaque choix : contexte, retenu, écarté, réversibilité.*

Fiche : `27-3-profil-apres-apercu.md`. Issue #626 (P2).

### C-27-3-1 — Changer de profil **relance** l'aperçu (plutôt que bloquer la confirmation)

- **Contexte** : l'issue laisse deux options — relancer l'aperçu, ou rendre la confirmation impossible tant qu'il
  n'est pas recalculé.
- **Retenu** : relancer aussitôt (AC1). Le geste de l'utilisateur dit ce qu'il veut voir ; l'aperçu coûte une
  requête sans écriture.
- **Écarté** : bloquer *Confirmer* avec un bouton « Recalculer » — un clic de plus sans information, et un état
  « aperçu périmé mais affiché » que l'utilisateur peut mal lire.
- **Réversible** : oui (écran seul).

### C-27-3-2 — Le déclencheur est le geste (`onChange`), jamais un `$effect` sur `bankProfileId`

- **Contexte** : `bankProfileId` est aussi affecté par le code (présélection KF #70, `:135-142`).
- **Retenu** : relance depuis le gestionnaire du sélecteur ; la présélection n'en déclenche aucune (AC4, mutation M2).
- **Écarté** : `$effect` — relancerait l'aperçu après chaque présélection (deux appels par dépôt, boucle possible).
- **Réversible** : oui.

### C-27-3-3 — Concurrence côté écran : numéro de séquence, sans anti-rebond ; sélecteurs actifs pendant l'aperçu, désactivés pendant la confirmation

- **Contexte** : certains navigateurs émettent `change` à chaque flèche sur un `<select>` fermé ; un compte changé ou *Annuler* pendant
  un aperçu en vol laissait déjà une réponse tardive s'afficher.
- **Retenu** : la dernière demande gagne, les réponses périmées sont ignorées (AC3) ; pendant une confirmation, les
  deux sélecteurs sont `disabled` (un changement à ce moment ne peut plus rien corriger).
- **Écarté** : anti-rebond (délai arbitraire, test plus fragile) ; désactiver les sélecteurs pendant l'aperçu
  (perte du focus clavier à chaque flèche).
- **Réversible** : oui.

### C-27-3-4 — Pendant le recalcul : ancien aperçu retiré, zone de dépôt masquée, texte « Recalcul… », cases remises à zéro

- **Contexte** : laisser l'ancien aperçu visible pendant le calcul montre des données qui ne seront pas importées ;
  la zone de dépôt (affichée dès que `preview` est nul) permettait un second dépôt concurrent.
- **Retenu** : AC2. Les cases de confirmation sont remises à zéro à chaque recalcul (prolonge F1 Pass 3).
- **Écarté** : ancien aperçu grisé (même risque de lecture, plus de code) ; conserver les cases cochées quand
  l'avertissement « survit » au recalcul (identité d'un avertissement difficile à établir, confirmation implicite).
- **Réversible** : oui.

### C-27-3-5 — Après un aperçu en échec, changer de profil relance avec le fichier retenu

- **Contexte** : aujourd'hui un refus (signe, aucun profil reconnu) oblige à redéposer ; c'est précisément le cas où
  l'on veut essayer un autre profil.
- **Retenu** : AC1-bis (sauf `BANK_ACCOUNT_REQUIRED`, où aucun fichier n'est retenu).
- **Écarté** : ne relancer que si un aperçu est affiché (lecture littérale de l'issue) — laisse le cas le plus utile
  de côté.
- **Réversible** : oui.

### C-27-3-6 — La confirmation part d'un instantané de l'aperçu affiché

- **Contexte** : `buildFlags()` lit l'état courant du sélecteur.
- **Retenu** : AC5 — profil **résolu** par l'aperçu, empreinte, fichier, compte, figés à la réception ; pour un CAMT,
  aucun `bankProfileId` (le serveur l'ignore déjà).
- **Écarté** : continuer à lire l'état courant en comptant sur la relance — un seul oubli et le défaut revient.
- **Réversible** : oui.

### C-27-3-7 — Défense serveur par **empreinte sans état** (SHA-256), ni table, ni jeton signé

- **Contexte** : l'issue demande si la confirmation peut porter une empreinte pour qu'un client périmé n'importe pas
  avec un mapping jamais vu.
- **Retenu** : l'aperçu rend `previewFingerprint`, la création la recalcule sur ses propres entrées et compare
  (AC6-AC8). Pas de migration, pas d'expiration, pas de nettoyage.
- **Écarté** : jeton stocké (migration, expiration, ne ferme rien de plus) ; HMAC (la menace est le client périmé,
  pas le client malveillant, qui a le droit d'importer) ; `updated_at` du profil (résolution à la seconde : deux
  modifications dans la même seconde passent).
- **Réversible** : oui (champ ajouté, facultatif).

### C-27-3-8 — Contenu de l'empreinte : domaine versionné, société, compte (id + IBAN normalisé), hash du fichier, format, profil résolu complet hors horodatages

- **Contexte** : il faut couvrir tout ce qui change la lecture du fichier, et rien de ce qui n'y touche pas.
- **Retenu** : AC7. Le profil par **déstructuration exhaustive** de `BankProfile` (un champ ajouté casse la
  compilation). Renommer un profil (`bank_name`) entre l'aperçu et la confirmation rend donc `409` : conséquence
  assumée (rare, et suivie d'un recalcul automatique).
- **Écarté** : seulement les champs « de mapping » — liste à tenir à la main, qui se périme en silence (le motif
  « inventorier les non-résolus plutôt qu'énumérer les formes » du CLAUDE.md) ; sérialiser le `CsvProfile` de
  `kesh-import` (crate publiable, on n'y ajoute pas de dérivation pour cet usage).
- **Réversible** : oui (balise `v1` pour une évolution).

### C-27-3-9 — Empreinte **facultative** à la création

- **Contexte** : `POST /api/v1/bank-imports` est ouvert aux clés d'API (`docs/api-external.md` § 7 : toute route
  `/api/v1/*` de l'interface est consommable) ; les tests d'intégration créent sans aperçu.
- **Retenu** : absente → comportement actuel ; l'écran l'envoie toujours. Angle mort assumé D12 (client sans
  empreinte, dont un onglet chargé avant la mise à jour), écrit dans `docs/api-external.md`.
- **Écarté** : obligatoire — rupture d'API pour une protection qui vise l'écran.
- **Réversible** : oui (rendre obligatoire plus tard serait une rupture, à annoncer).

### C-27-3-10 — `409 BANK_IMPORT_PREVIEW_STALE` ; forme invalide ou doublon → `400` ; précédence après les refus antérieurs à la résolution du profil

- **Contexte** : les refus métier de l'import sont en `422` ; un conflit d'état entre lecture et écriture est un
  `409` ailleurs dans Kesh (conflits de version).
- **Retenu** : `409` ; comparaison CSV après `resolve_csv_profile`, sur le profil lu dans la transaction (aucune
  fenêtre avec le parse) ; `422 BANK_IMPORT_DUPLICATE_FILE` non confirmé reste prioritaire (AC8, I10).
- **Écarté** : `422` (ce n'est pas un défaut du fichier) ; déplacer la résolution du profil avant la détection de
  doublon (change l'ordre des refus existants).
- **Réversible** : oui.

### C-27-3-11 — Sur `409`, l'écran recalcule et l'annonce, sans reconfirmer seul

- **Contexte** : profil modifié dans un autre onglet (D3).
- **Retenu** : AC9 — recalcul automatique + avis `role="status"` ; l'utilisateur reconfirme.
- **Écarté** : afficher l'erreur seule (l'utilisateur devrait deviner qu'il faut redéposer) ; reconfirmer
  automatiquement après recalcul (importerait sans que l'utilisateur ait vu le nouvel aperçu — le défaut même).
- **Réversible** : oui.

### C-27-3-12 — L'état de la base (doublons, fichier déjà importé) reste hors empreinte

- **Contexte** : un autre import entre l'aperçu et la confirmation change les doublons.
- **Retenu** : la création les recalcule déjà et refuse un fichier déjà importé non confirmé ; les lignes doublons
  sont ignorées par défaut. Les inclure ferait rougir l'empreinte pour un événement que la création traite déjà.
- **Réversible** : oui.

### C-27-3-13 — CAMT.053 : même mécanisme, profil absent de l'empreinte ; changer de profil relance aussi

- **Contexte** : le profil n'a pas d'effet sur un CAMT.
- **Retenu** : empreinte rendue et vérifiée pour les deux formats (compte et fichier comptent) ; relance uniforme
  côté écran (une requête sans effet, mais aucune branche par format à tenir).
- **Écarté** : ne relancer que pour un CSV — exigerait de connaître le format côté écran avant l'aperçu.
- **Réversible** : oui.

### C-27-3-14 — Manuel : l'encadré C-26-3-15 est réécrit ; les « redéposez le fichier » du refus de signe restent

- **Contexte** : `user-manual.tex:1743`, `:1784` et le message serveur `amount-sign-refused-profile` disent de régler
  le profil puis de redéposer.
- **Retenu** : inchangés — le profil se modifie dans *Administration*, hors de l'écran d'import, et re-choisir la
  même valeur du sélecteur n'émet aucun `change`. Seuls `:1697-1698`, `:1794-1796` et `:1801-1807` changent (AC12).
- **Réversible** : oui.

### C-27-3-15 — Aucune migration, aucun bump de version

- **Retenu** : AC15 ; la release 0.13.2 fera le bump Cargo. Pas de `min_required` (aucun changement de schéma).

### C-27-3-16 — L'empreinte vit dans un module neuf, hors de `bank_imports.rs`

- **Contexte** : la 27-1 modifie `bank_imports.rs` et `errors.rs`.
- **Retenu** : `routes/bank_import_fingerprint.rs` (fonction + tests unitaires) ; dans `errors.rs`, variante et
  mapping placés près de `BankCsvProfileNotFound`, loin des hunks de la 27-1.
- **Réversible** : oui.

### C-27-3-17 — E2E : contenu dérivé en mémoire de la fixture UBS synthétique

- **Contexte** : `ubs_synthetique.csv` est importée par un test 26-1 de la même spec : la réutiliser telle quelle
  déclenche « fichier déjà importé » et l'ignorance des 11 lignes en doublon.
- **Retenu** : `buffer` de `setInputFiles`, année des dates remplacée (2027 pour E1, 2028 pour E2), nom de fichier
  propre. Aucune donnée réelle (relevé de Guy hors de portée, fixture synthétique du dépôt).
- **Écarté** : nouveau fichier de fixture (copie qui dérive de l'original).
- **Réversible** : oui.

### C-27-3-18 — Pas de trace d'audit pour le refus `409`

- **Retenu** : comme les autres refus de l'import (aucune écriture, aucune trace) ; I4 vérifie l'absence.
- **Réversible** : oui.

### C-27-3-19 — Trois clés i18n seulement ; le « Chargement… » en dur reste hors périmètre

- **Contexte** : `BankImportUpload.svelte:305` porte un « Chargement… » non traduit (et d'autres libellés en dur du
  même écran : « Ligne », « Code », « Valeur », les intitulés de l'aperçu).
- **Retenu** : ne pas les traiter ici (dette d'i18n de l'écran, distincte du défaut) ; trois clés neuves (AC10),
  `sitesTotal` +2 (AC11).
- **Réversible** : oui. **À signaler à l'orchestrateur** : issue P4 éventuelle pour les libellés en dur de l'écran
  d'import.
