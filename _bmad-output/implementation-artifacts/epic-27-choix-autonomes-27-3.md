# Epic 27 — choix autonomes de la Story 27-3 (à fusionner dans `epic-27-choix-autonomes.md`)

*Écrit le 2026-10-12 par l'agent de spécification (Opus 5.5), en autonomie ; C-27-3-20 à 27 ajoutés le même jour
par l'agent de remédiation de la validation P1, C-27-3-28 à 34 par celui de la validation P2. Le registre de l'epic vit sur la branche
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
- **Précisé à la validation P1** : *Annuler* n'est jamais rendu pendant un aperçu (il vit dans le bloc
  `{#if preview}`) — seul le changement de compte invalide un aperçu en vol ; *Annuler* est désactivé avec les
  sélecteurs pendant la confirmation (C-27-3-21).
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

- **Contexte** : l'issue #626 demande un écran juste (« changer de profil relance l'aperçu, ou rend la confirmation
  impossible tant que l'aperçu n'a pas été recalculé ») ; elle ne parle pas du serveur. La story **ajoute** une
  défense serveur, parce qu'un écran corrigé reste exposé à un profil modifié dans un autre onglet (D3) et qu'un client
  périmé ne doit pas importer avec un mapping jamais vu. *(Contexte reformulé à la validation P2, R13 : la première
  rédaction attribuait cette demande à l'issue, qui ne la contient pas.)*
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
- **Révisé à la validation P1 (2026-10-12)** : sur consigne de l'orchestrateur (le nombre de bugs doit baisser à la
  release, pas de nouvelle issue), les libellés en dur de l'écran d'import sont **traduits dans cette story** —
  C-27-3-25. Ce choix-ci reste pour l'historique ; la proposition d'issue P4 est retirée.

## Validation P1 (2026-10-12) — choix de remédiation

### C-27-3-20 — `reset()` éteint les indicateurs et l'avis, en plus d'invalider la séquence

- **Contexte** : F1 (HIGH) / R3. Une réponse périmée n'éteint plus l'indicateur (AC3) ; si `reset()` (changement de
  compte pendant un aperçu) ne le fait pas non plus, `isPreviewing` reste vrai, la zone de dépôt reste masquée
  (AC2) et l'écran est bloqué jusqu'au rechargement.
- **Retenu** : `reset()` remet `isPreviewing` et `isConfirming` à faux et efface l'avis de l'AC9 ; V8 asserte
  l'indicateur absent et la zone présente, dès le changement et après la réponse tardive ; mutation M19.
- **Écarté** : éteindre l'indicateur dans le `finally` de toute réponse (rallume *Confirmer* pendant qu'un aperçu
  plus récent est en vol — le piège déjà écrit aux Dev Notes).
- **Réversible** : oui.

### C-27-3-21 — Pas d'*Annuler* pendant un aperçu ; *Annuler* désactivé pendant la confirmation

- **Contexte** : F2 / R1 — D11 et V10 supposaient un *Annuler* pendant un aperçu en vol, or le bouton n'est rendu
  qu'avec un aperçu (`:354`) que l'AC2 retire. F5 / R6 — pendant une confirmation, *Annuler* reste actif ; suivi d'un
  `409`, il ferait réapparaître l'aperçu après l'annulation.
- **Retenu** : D11 réécrit « inatteignable » (la sortie pendant un aperçu est le changement de compte, D5) ; V10 et la
  mention de V10 en M3 retirées, V10 réaffecté (C-27-3-22) ; *Annuler* `disabled` pendant la confirmation, comme les
  sélecteurs (AC3, D15, V9, M8).
- **Écarté** : rendre un *Annuler* dans l'état de chargement (un site d'invalidation de plus à tester, pour un
  aperçu qui dure une requête sans écriture) ; garder *Annuler* actif et garder la relance `409` par un jeton de
  séquence capturé au début de la confirmation (deux mécanismes là où un `disabled` suffit : une fois les trois
  contrôles désactivés, aucun geste **sur l'écran** n'invalide plus la confirmation).
- **Précisé à la validation P2** (R14) : la première rédaction disait « plus aucun geste ne peut invalider la
  confirmation » ; **quitter la page** en reste un (démontage du composant, D19) — inoffensif, assumé (C-27-3-34).
- **Réversible** : oui.

### C-27-3-22 — M6 est tuée par un test CAMT, et non par un test serveur

- **Contexte** : F3 / R2 — M6 renvoyait à I5, test Rust qui ne peut pas tuer une mutation du Svelte (erreur de
  catégorie), et à une variante de V1 contradictoire avec l'AC2. Sélecteur et instantané ne divergent que pour un
  CAMT (l'instantané n'envoie aucun profil).
- **Retenu** : V10 réécrit — aperçu sans `csvProfileMatch`, sélecteur sur A, *Confirmer* → `createBankImport` sans
  `bankProfileId`.
- **Écarté** : déclarer M6 équivalente (elle ne l'est pas : le CAMT la distingue).
- **Réversible** : oui.

### C-27-3-23 — M16 en deux ; la fenêtre transactionnelle est tenue par construction, non par un test

- **Contexte** : F4 / R4 — la première forme de M16 (comparaison avant la résolution, profil relu hors transaction)
  n'était tuée par aucun test, et la précédence de l'AC8 n'était testée qu'à moitié.
- **Retenu** : I11 (profil supprimé → `404`, non `409`) et I12 (compte inconnu → `404`) ; M16a (comparaison déplacée
  avant la résolution, un profil supprimé rend `409`) → I11 ; M16b (empreinte sur l'id demandé) → I6 — **faux, corrigé à
  la validation P2** : I6 ne tue pas M16b, M16b → I6-bis (C-27-3-29). La variante
  « relu hors transaction mais présent » est écrite **équivalente** en test déterministe, non comptée, tenue par
  construction (une seule variable `profile` comparée puis parsée).
- **Écarté** : un test de concurrence (injection d'un `UPDATE` entre lecture et parse : outillage absent, test
  fragile pour une fenêtre que le code ferme par construction).
- **Réversible** : oui.

### C-27-3-24 — Précédence asymétrique entre CSV et CAMT : conservée, écrite et testée

- **Contexte** : F7 / R11 — CSV : `422 DUPLICATE_FILE` avant `409` ; CAMT : `409` avant `422 DUPLICATE_FILE` (la
  détection de doublon CAMT se fait dans la transaction, la comparaison avant).
- **Retenu** : conserver ; écrire l'ordre à `docs/api-external.md` (AC13) ; I10 teste le cumul pour les deux formats.
- **Écarté** : déplacer la comparaison CAMT dans la transaction après la détection de doublon (aligne l'ordre, mais
  ouvre une transaction pour rien sur un import refusé) ; aucune incidence fonctionnelle à aligner.
- **Réversible** : oui.

### C-27-3-25 — Les libellés en dur de l'écran d'import sont traduits dans cette story

- **Contexte** : C-27-3-19 les laissait hors périmètre ; l'orchestrateur demande de les traduire ici, dans les quatre
  locales, sans ouvrir d'issue.
- **Retenu** : AC17 — périmètre l'écran `/bank-import` (page et trois composants), inventaire clos H1-H11 avec ses
  non-résolus assumés (symboles, codes bruts, gabarit `bank-import-info-*`) ; textes fr-CH repris à l'identique ; deux
  clés globales réutilisées (`common-loading`, `error-unexpected`, autorisées par `lint-i18n-ownership`), 21 clés
  neuves, une clé partagée entre l'aperçu et la liste (`bank-import-labels-period`) ; `sitesTotal` 2032 → 2063, 2072
  à la fusion avec la 27-1.
- **Écarté** : traduire aussi la fiche d'import `/bank-import/{id}` (une quinzaine de libellés en dur) — autre écran,
  modifié par la 27-1 au bloc adjacent : conflit certain. **Signalé à l'orchestrateur**, qui décide (après la fusion
  de la 27-1 : une story ou une issue). Reformuler les textes fr (« Solde d'ouverture », « relevé(s) ») — hors du
  défaut, et ferait mentir des captures.
- **Réversible** : oui.

### C-27-3-26 — `handleFileInput` vide `input.value`

- **Contexte** : R12 — l'`<input type=file>` n'est jamais vidé ; après un refus, la zone reste montée et re-choisir
  le même fichier n'émet pas `change`, alors que le manuel et le message serveur disent « redéposez le fichier ».
  Constat raisonné, non exécuté (la validation est en lecture seule) ; l'AC2 démonte la zone pendant tout aperçu,
  ce qui le ferme en pratique, mais par accident de rendu.
- **Retenu** : AC16 — une ligne, `input.value = ''` après lecture du fichier ; V12 (espion du setter, `user-event`
  absent) ; mutation M17.
- **Écarté** : s'en remettre au démontage de l'AC2 (une évolution du rendu rouvrirait le défaut en silence).
- **Réversible** : oui.

### C-27-3-27 — Le titre et le CHANGELOG disent « depuis l'écran »

- **Contexte** : R9 — « le serveur refuse un import jamais prévisualisé » sur-promet : un client sans empreinte
  importe (D12).
- **Retenu** : titre « … refuse un import dont le profil a changé depuis l'aperçu » ; CHANGELOG « depuis l'écran » et
  « facultatif pour un client d'API ». Le nom du fichier de la fiche ne change pas.
- **Réversible** : oui.

### C-27-3-28 — L'avis du `409` est neutre : « Le profil, le fichier ou le compte a changé »

- **Contexte** : validation P2, F1 = R2 — l'avis prescrit (« Le profil a été modifié depuis la prévisualisation… »)
  imputait tout `409` au profil. Or, depuis l'écran, l'instantané fige l'objet `File` et l'id du compte, et un `409`
  peut venir du contenu du profil (D3), de l'IBAN du compte modifié ailleurs (D7) ou des octets du fichier modifié sur
  le disque (D18) ; pour un CAMT, le profil n'est pas dans l'empreinte, l'avis y était faux à chaque fois. Le message
  serveur, lui, disait déjà « Le profil, le fichier ou le compte a changé ».
- **Retenu** : avis aligné sur le message serveur, dans les quatre locales et le repli (AC9, AC10) ; le texte de
  recalcul perd « avec le profil choisi » (« Recalcul de la prévisualisation… »), faux lui aussi après un `409` sur un
  CAMT ; le manuel (AC12) cite le profil, le compte et le fichier.
- **Écarté** : un avis par cause (l'écran ne sait pas laquelle des entrées a changé — il faudrait que le serveur
  rende le détail de l'empreinte, ce qui la rendrait structurée pour un gain d'affichage) ; un avis « profil » pour
  un CSV et « compte » pour un CAMT (plus de code, et faux pour le fichier modifié).
- **Réversible** : oui (textes seuls).

### C-27-3-29 — M16b est tuée par I6-bis, création sans `bankProfileId`

- **Contexte** : validation P2, F2 = R1 (régression de la remédiation P1) — M16b (« empreinte sur l'id demandé au
  lieu du profil résolu ») était affectée à I6, où la création envoie l'id résolu : demandé et résolu y sont le même
  profil, la mutation est équivalente dans I6, qui reste vert.
- **Retenu** : I6-bis — aperçu et création sur *Auto-détection*, aucun id des deux côtés, un seul profil concordant,
  empreinte de l'aperçu → `201` ; sous la mutation, l'empreinte de la création porte `null` à la place du profil →
  `409`. M16b réécrite précisément (`fields.bank_profile_id.and_then(<relecture>)`, `null` sans id). I6 reste, pour
  l'AC8 (auto-détection puis explicite). Helper existant `build_multipart_with_text_fields` (F8).
- **Écarté** : muter aussi l'aperçu (la mutation ne porterait plus sur le site de création qu'elle doit éprouver).
- **Réversible** : oui (tests).

### C-27-3-30 — La garde de séquence est éprouvée sur une réponse périmée en vol, et sur une réponse en erreur

- **Contexte** : validation P2, R3 (régression de la remédiation P1, V3 réécrit) — dans l'ordre C, A, B, la réponse
  la plus récente arrive la première : aucune réponse périmée n'arrive pendant qu'un aperçu plus récent est en vol, si
  bien qu'un `finally` qui éteint l'indicateur sans condition survit ; et V3 / V8 n'employaient que des réponses
  réussies, si bien qu'une garde posée sur la seule branche de succès survivait aussi.
- **Retenu** : V3 dans l'ordre A, C, B (indicateur présent et aucun aperçu après A) ; variante rejetée de V8 (ni
  erreur, ni indicateur) ; mutations M21 et M22 ; T3 écrit qu'une réponse périmée, succès ou erreur, ne touche ni
  `preview`, ni l'erreur, ni `isPreviewing`.
- **Écarté** : un test par permutation des trois réponses (six ordres) — A, C, B est le seul qui fasse arriver une
  réponse périmée avant la plus récente **et** une autre après.
- **Réversible** : oui (tests).

### C-27-3-31 — L'avis s'efface en trois sites, chacun tenu par un test

- **Contexte** : validation P2, R4 — l'AC9 promet que l'avis disparaît au geste suivant, mais la branche `409` ne
  passe pas par `reset()` et T3 ne prescrivait que l'effacement par `reset()` ; aucun test ni mutation ne tenait la
  disparition.
- **Retenu** : trois sites prescrits (T3) — `reset()` (compte, *Annuler*), `runPreview` pour toute raison autre que
  `'stale'` (dépôt, profil), `handleConfirm` dès le clic ; V6-bis (un rendu par geste) ; mutation M20.
- **Écarté** : un `$effect` qui efface l'avis à tout changement de `preview` (effacerait l'avis à l'arrivée même du
  recalcul qu'il annonce) ; un délai d'expiration (arbitraire, l'utilisateur peut ne pas l'avoir lu).
- **Réversible** : oui.

### C-27-3-32 — L'annotation « (auto-détecté) » n'apparaît que pour un aperçu `autoMatched`

- **Contexte** : validation P2, F9 — `BankImportUpload.svelte:252` alimente l'annotation par
  `csvProfileMatch.profileId`, que `preview_csv` pose toujours pour un CSV : un profil choisi avant le dépôt est déjà
  annoté « (auto-détecté) » à tort, et l'AC1, qui relance l'aperçu à chaque choix, rendrait l'erreur systématique.
- **Retenu** : `autoMatchedId` vaut l'id seulement si `csvProfileMatch.autoMatched` (AC4) ; V1 (pas d'annotation sur
  B), V4 (annotation sur le présélectionné) ; mutation M23. La phrase du manuel (« mention « auto-détecté » » sous
  *Auto-détection*) reste vraie et devient exacte.
- **Écarté** : laisser tel quel (défaut préexistant, mais que la story aggrave) ; une issue séparée (correctif d'une
  ligne dans un fichier que la story réécrit).
- **Réversible** : oui.

### C-27-3-33 — Sur `409`, `isConfirming` s'éteint avant le recalcul

- **Contexte** : validation P2, F5 — si la branche `409` attend le recalcul (`await runPreview(…)`) avant le
  `finally` qui éteint `isConfirming`, les sélecteurs restent désactivés pendant tout le recalcul, contre l'AC3
  (« pendant un aperçu, les sélecteurs restent actifs »), et un recalcul qui traîne fige l'écran.
- **Retenu** : `isConfirming = false` puis `void runPreview(…, 'stale')` (AC9, T3) ; assertion dans V6 (sélecteurs
  actifs pendant le recalcul retenu) ; mutation M24.
- **Écarté** : garder l'`await` et désactiver délibérément pendant le recalcul (aucune raison : le recalcul est un
  aperçu comme un autre, protégé par la séquence).
- **Réversible** : oui.

### C-27-3-34 — D18 (fichier modifié sur le disque) et D19 (navigation pendant la confirmation) : écrits, sans code neuf

- **Contexte** : validation P2, F3 et R14 — l'inventaire D1-D17 se disait clos sans le fichier source modifié entre
  l'aperçu et la confirmation (l'objet `File` est lu paresseusement) ni le démontage du composant pendant une
  confirmation ; F3 relève aussi la suppression d'un import par la 27-1, de la classe D13.
- **Retenu** : D18 fermé par l'empreinte existante (hash du fichier, AC7-4 → `409`, recalcul, avis neutre
  C-27-3-28) ; D19 inoffensif et assumé — une requête d'aperçu sur un composant démonté, sans effet visible ni
  écriture —, sans garde de démontage ; D13 nomme la suppression d'un import.
- **Écarté** : une garde de démontage (`onDestroy` qui invalide la séquence) — un site de plus à tester pour une
  requête sans écriture ; relire le fichier avant la confirmation pour comparer côté client (le serveur le fait déjà).
- **Réversible** : oui.
