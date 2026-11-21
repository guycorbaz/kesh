# Registre des choix autonomes — story 48-13 « Avant la vraie facture » (clé `e4813`)

Choix pris au développement (Opus 5.5, `bmad-dev-story`, 2026-11-18), sans question à Guy ni à l'orchestrateur. Chacun : contexte,
option retenue, autres options, réversibilité.

## C-e4813-1 — AC6 : les noms acceptés en un seul cas, plus exigeant

- **Contexte** : l'AC6 demande d'accepter un nom de 70 caractères, un nom avec « & », « é » et « ’ » (U+2019), et un nom entouré
  d'espaces rogné.
- **Retenu** : un seul `#[sqlx::test]` (`coordinates_accepts_a_70_character_name_and_trims_it_1226`) avec un nom de **70 caractères qui
  porte « & », « é » et « ’ »**, envoyé entouré d'espaces, et relu en base rogné et entier. Les caractères multi-octets en font aussi
  la preuve d'un compte en caractères, non en octets (un compte en octets dépasserait 70).
- **Écarté** : trois tests séparés (trois bases éphémères, sans gain de preuve).
- **Réversible** : oui, découpage mécanique.

## C-e4813-2 — AC6, AC7 : les refus en boucle dans un test par famille

- **Contexte** : quatre refus de nom (71 caractères, « 张 », U+00AD seul, « prénom nom » composé à 71) et deux refus d'adresse (rue
  « ʼ » U+02BC, localité « 张 »).
- **Retenu** : un test par famille, les cas en boucle ; chaque cas vérifie `400 VALIDATION_ERROR`, le message, la société inchangée
  (nom, prénom, nom de famille, cinq colonnes d'adresse) et l'étape toujours à 5. Un refus n'écrit rien : les cas ne s'influencent pas.
- **Limite** : un rouge s'arrête au premier cas qui rougit (constaté au rouge et sous M6 : le cas « 71 caractères »).
- **Écarté** : un test par cas (six bases éphémères).
- **Réversible** : oui.

## C-e4813-3 — AC8 a : le test d'écran dans `onboarding-motif.test.ts`

- **Contexte** : la fiche dit « `onboarding-screens.test.ts` ou voisin ».
- **Retenu** : `onboarding-motif.test.ts`, qui porte déjà les refus du serveur de l'étape des coordonnées (S7, #809) et leurs
  simulations ; un cas neuf y vérifie le nom de 71 caractères envoyé entier, le toast du serveur, la saisie gardée, l'écran resté aux
  coordonnées, et l'absence de `maxlength` sur `#coord-name`.
- **Réversible** : oui.

## C-e4813-4 — AC3 : le client choisi en E2E par la frappe de son nom

- **Contexte** : l'AC1 (Vitest) choisit **sans frappe** pour que M3 rougisse (une frappe avance déjà le compteur, D10). En E2E, la base
  est partagée par les specs : la liste ouverte au focus (recherche vide, 50 premiers) ne contient pas sûrement le client du test.
- **Retenu** : `fill` du nom unique puis clic sur l'option. La preuve du rouge visée en E2E est **M1** (garde du `.then` retirée), que
  la frappe n'empêche pas : rejouée, elle rougit à la ligne 93 (le client d'origine revient). M3 reste prouvée en Vitest.
- **Écarté** : un choix au clavier sans frappe (flèche sur la liste du focus) : dépend du nombre de contacts de la base partagée.
- **Réversible** : oui.

## C-e4813-5 — Manuel : « la version~0.20.0 » écrit dès la story

- **Contexte** : D7 dit que le numéro de la phrase réécrite (`user-manual.tex:1522`) est « posé à l'intégration » ; la mémoire de Guy
  demande un numéro de release, jamais « Epic N », dans les manuels.
- **Retenu** : « la version~0.20.0 » (forme des phrases voisines, « la version~0.17.4 »), aux deux sites (`:219`, `:1522`). La release
  0.20.0 est fixée (plan de la release, sortie le 2026-11-27).
- **Si la story sortait de la 0.20.0** : les deux occurrences se reprennent (`git grep -n "version~0.20.0" -- docs/manual`).
- **Réversible** : oui.

## C-e4813-6 — Script des mutations : remplacement par motif vérifié unique

- **Contexte** : consigne « Edit seulement » pour le code ; les scripts restent permis pour les mutations (motif vérifié unique,
  restauration `git restore` + `touch`).
- **Retenu** : `kesh-gate-logs/e4813/mut/muter.sh` (refuse un fichier non commité, motif compté exactement une fois, restauration
  vérifiée par `git diff --quiet`). Aucune autre écriture de code par script.
- **Réversible** : sans objet.
