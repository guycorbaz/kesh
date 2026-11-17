# Epic 44 — registre des choix autonomes de la story 44-1a (clé `44-1`)

Choix faits au développement de la 44-1a (2026-11-17, Opus 5.5, base `4b3a20b0b`), hors de ce que la fiche, le fichier
d'epic et le registre `e36-choix-autonomes-36-12.md` (C-36-12-1, -3 à -10) tranchent déjà. Chaque choix : contexte, option
retenue, options écartées, réversibilité.

### C-44-1-1 — Le message de huit lignes des fixtures P3 : mots de 17 et 16 `W` en alternance

- **Contexte** : la fiche (AC5) décrit un « message de 140 caractères en 8 lignes (mots de 17 `W`) ». Huit mots de 17 `W` et
  leurs sept espaces font 143 caractères, au-delà des 140 admis (`USTRD_MAX`, `validation.rs:17`) ; sept mots de 17 et un de
  14 font 140, mais le dernier tient alors à côté du septième (45,29 + 0,78 + 37,30 = 83,4 ≤ 87) : sept lignes.
- **Retenu** : quatre paires 17 + 16 `W`, 139 caractères : deux mots voisins font au moins 45,29 + 0,78 + 42,63 = 88,7 mm
  > 87, une ligne par mot, huit lignes (relevé par `kesh-gate-logs/e44/outils/coupe_afm.py`).
- **Écartées** : (a) huit mots de 17 `W` : refusés par la validation ; (b) un autre caractère que `W` : la fiche raisonne
  en `W` partout.
- **Réversible** : oui (fixtures de test et de capture seulement).

### C-44-1-2 — La mise en page pure : une `Colonne` par palier, le dernier palier en repli

- **Contexte** : D4 demande une fonction pure, sans erreur, à paliers ordonnés où la 44-1b insère les siens.
- **Retenu** : `mise_en_page(data, i18n, devise, montant) -> MiseEnPage` ; chaque palier construit sa `Colonne` (textes
  posés de haut en bas, pas nommés) ; on prend le premier palier qui tient parmi tous **sauf le dernier**, sinon le dernier
  (`split_last`, `unwrap_or_else`) — aucun `unwrap` ni `expect` atteignable par une entrée ; un `debug_assert!` dit que le
  dernier tient. Les paliers sont des `enum` avec leur liste ordonnée (`PALIERS_RECU`, `PALIERS_PAIEMENT`). Le montant
  absent reste refusé avant la mise en page (`InvalidAmount`, comme avant), pour qu'elle ne rende pas de `Result` (AC7).
- **Écartées** : (a) une mise en page qui dessine en calculant (l'ancienne) : impossible d'essayer un palier sans
  l'imprimer ; (b) mesurer la hauteur à part puis dessiner : deux calculs qui peuvent diverger — c'est ce que l'AC6 interdit.
- **Réversible** : oui (interne au module).

### C-44-1-3 — `raccourcir` retire l'espace de fin avant les points

- **Contexte** : D3 dit seulement « le dernier caractère conservé est celui qui laisse la place des trois points ».
- **Retenu** : le début le plus long, **puis** `trim_end`, puis « ... » (« DE-WWW… » plutôt que « DE-WWW ... ») ; une ligne
  qui tient sort entière, sans points ; les espaces multiples se réduisent comme dans `couper_ligne`.
- **Écartée** : garder l'espace (« Rue ... ») : lu comme un mot manquant.
- **Réversible** : oui.

### C-44-1-4 — Les lignes d'adresse portent leur rôle

- **Contexte** : R1 doit retirer « toutes les lignes issues de la rue » ; après `lignes_adresse`, une ligne ne dit plus
  d'où elle vient (la rue peut être vide, et alors absente).
- **Retenu** : `lignes_adresse_par_role` rend `(Role, String)` (Nom, Rue, Localité, LigneK) ; la rue se retire **avant** la
  coupe, donc toutes ses lignes coupées partent ensemble ; `une_rue` vaut si l'une des deux adresses a une ligne `Rue` non
  vide. `lignes_adresse` reste, sans les rôles, pour ses trois tests (`#[cfg(test)]`).
- **Écartée** : reconnaître la rue par sa position (deuxième ligne) : faux pour une adresse sans rue.
- **Réversible** : oui.

### C-44-1-5 — Le caractère le plus large de la propriété est `@`, calculé

- **Contexte** : l'AC8 exige le caractère le plus large « calculé sur les sorties de `texte_imprime` sur le jeu admis, non
  écrit en dur ». La fiche raisonne en `W` (944).
- **Retenu** : le maximum sur U+0020 à U+2100 admis par `est_caractere_qr` : `@` (1015 millièmes d'em), devant `W`, `Æ`, `Œ`
  (1000) ; la propriété porte sur lui (80 cas : cinq formes de mots × pays CH/DE × avec ou sans référence × message 0/140 ×
  type S/K). Les fixtures nommées gardent le `W` de la fiche.
- **Écartée** : `W`, écrit en dur : contraire à l'AC8, et moins large que `@`.
- **Réversible** : oui.

### C-44-1-6 — Le relevé `pdftotext -bbox` lit le PDF sur son entrée standard

- **Contexte** : `pire_cas_boites_independantes` (AC8) passe par poppler ; un fichier temporaire serait à créer puis à
  retirer.
- **Retenu** : le PDF est écrit sur l'entrée de `pdftotext -bbox - -`, la sortie lue en mémoire ; aucune écriture disque. La
  hauteur de la page est lue à la `MediaBox` (842 pt : A4 ; la section de paiement est le bas de la page).
- **Réversible** : oui.

### C-44-1-7 — Une fixture « R1 sur le seul débiteur » à neuf lignes

- **Contexte** : l'AC5 veut une fixture où seul le débiteur déborde ; la première composée (localité « Rorschacherberg-
  Wartensee-Staad ») tenait sur une ligne, huit lignes au total : R0.
- **Retenu** : localité « Rorschacherberg-Wartensee-Buechen » (33 caractères ≤ 35), 55,37 mm avec le NPA : le NPA reste
  seul ; débiteur de six lignes (nom 2, rue 2, localité 2), créancier ordinaire de trois : neuf, R1 (relevé par
  `outils/coupe_afm.py`, confirmé par `la_cascade_par_palier`).
- **Réversible** : oui (fixture de test).
