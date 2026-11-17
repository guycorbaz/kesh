# Registre des choix autonomes — voie `v0190c` (release 0.19.0)

*Créé le 2026-11-17 à la remédiation de la passe 1 de validation des fiches `v0190-correction` (#1162) et `v0190-nom-societe` (#1097) ;
les choix C-v0190c-1 à 10 sont repris des fiches, les suivants naissent de la remédiation.*

## Fiche `v0190-correction.md` (#1162, `ContactPicker`)

- **C-v0190c-1** (D1) : le brouillon `draft` se jette au `blur` ; `query = draft ?? selected?.name ?? ''` ; un changement de `selected?.id` le jette. Écarté : résoudre le texte par une recherche au `blur` (faux rattachement muet) ; écarté : un message.
- **C-v0190c-2** (D2) : *Entrée* — (1) désignation + recherche courante terminée → choix ; (2) brouillon affiché → jamais d'envoi, la liste s'ouvre ; (3) champ non touché → le formulaire part.
- **C-v0190c-3** (D3) : la surbrillance ne vaut choix que désignée ; rendu par `CLASSE_DESIGNEE` / `CLASSE_SURVOL` réemployées.
- **C-v0190c-11** (D2, V1-6) : brouillon blanc + *Entrée* → brouillon jeté, liste fermée, client retenu réaffiché, envoi empêché ; l'*Entrée* suivante envoie (patron E5 de la 36-1). Troisième différence avec la 36-1.
- **C-v0190c-12** (D2, V1-1) : chaque frappe invalide toute recherche en vol ; `pending` ne se lève qu'à la réponse de la recherche du brouillon actuel ; au `blur` et au choix, anti-rebond annulé et `pending` à faux.
- **C-v0190c-13** (D3, V1-7) : flèche bas / haut ouvrent la liste fermée et désignent le premier / le dernier (patron `AccountAutocomplete`).

- **C-v0190c-16** (D1 à D3, validation P2 V2-2 à V2-7) : la frappe efface la désignation ; `loading` et `results` remis à zéro au `blur` et au choix ; réponse jugée sur le `draft` courant à son arrivée, échec = réponse ; le changement de `selected?.id` purge désignation, liste, `pending` ; sous-ligne IDE lisible (≥ 4,5:1) ; `preventDefault` sur `mousedown`.
- **C-v0190c-17** (AC3, V2-8) : le test de page *Réinitialiser* est exigé ; AC4 : lignes remplies avant la frappe, aucun `POST` parti, rouge constaté au développement ; jalon mercredi 18 h (la story sort de la release).

- **C-v0190c-19** (D1 à D3, remédiations des validations P3 à P6) : `searchSeq++` au changement de client (P3-1) ; « Chargement… » tant que `pending || loading` (P3-3) ; règle 2 (cas E3) : *Entrée* sur un brouillon rouvre et désigne si `pending` est faux (P3-4) ; `group-hover` réservé à l'option non désignée (P4-1, P5-1) ; la relance par la flèche reprend le texte de la frappe (P4-2) ; règle unique de désignation : par la saisie (brouillon non blanc, liste ouverte à l'arrivée) ou par une flèche, jamais par l'arrivée pour un brouillon nul (P5-2, P6-4) ; la direction de la relance est un paramètre de `runSearch`, capturé avec `seq`, appliqué seulement si `seq === searchSeq`, sans état du composant (P6-1) ; la désignation par la flèche l'emporte sur celle de la saisie (P6-3). Écarté : un état `arrowRelance` (survit à une recherche abandonnée).

## Fiche `v0190-nom-societe.md` (#1097, nom de la société)

- **C-v0190c-4** (D1) : route dédiée `PUT /api/v1/companies/current/name`, Admin-only ; écarté : étendre `address` ou `contact-details`.
- **C-v0190c-5** (D2) : contrôles du nom dans `validate_company_name_for_qr` (vide au sens de la QR-facture, 70 caractères, jeu de caractères), `400 VALIDATION_ERROR`, français littéral.
- **C-v0190c-6** (D3) : `first_name` / `last_name` ne bougent pas ; limite tracée par #1227 (acceptée par l'orchestrateur le 2026-11-17).
- **C-v0190c-7** (D4) : transaction, verrou optimiste, no-op sans audit, `company.updated` avec `{before, after}` limité au nom.
- **C-v0190c-8** (D5) : bloc d'édition du nom transposé du bloc e-mail ; vide refusé localement, longueur et jeu de caractères au message du serveur ; écarté : recopier le jeu de caractères dans le frontend.
- **C-v0190c-9** (D6) : les trois refus du PDF (replis Rust, quatre catalogues) renvoient à *Paramètres → Organisation* ; deux gardes inversées.
- **C-v0190c-10** (D7) : manuel utilisateur ; `.tex` seulement.
- **C-v0190c-14** (D5, V1-3) : la branche `nameEditing` de `syncOpenBlocks` ; AC4 : édition du nom ouverte, enregistrement d'un autre bloc, pas de 409 parasite.
- **C-v0190c-18** (validation P2 V2-N1 à V2-N6) : liste des effets du nom complétée (rapports, variable d'e-mail) ; refus de la route en français seul, limite assumée rattachée à #710 ; AC1 g-bis (société sans composants), AC4 (nom stocké de 71 caractères, cas de non-régression : la règle commune rogne déjà, aucun normaliseur ajouté, N3-1) ; version de la phrase du manuel posée à l'intégration ; plan de coupure tout ou rien, jalon mercredi 18 h.
- **C-v0190c-15** (D7, V1-5) : aucune route ajoutée à la note de montée de la 0.17.4 (`admin-manual.tex:2667`) ; la route se dit à la section « Passer à la 0.19.0 » ; si un manuel la porte, `MULTI_SOCIETE_ASSUME` reçoit un fragment et `MULTI_SOCIETE_TOTAL` passe de 22 à 23 (recompté).
