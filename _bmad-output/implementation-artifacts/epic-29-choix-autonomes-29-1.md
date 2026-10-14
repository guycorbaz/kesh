# Epic 29 — registre des choix autonomes, Story 29-1 (à verser par union dans `epic-29-choix-autonomes.md`)

*Fiche : `29-1-preparer-cloture.md`. Choix faits le 2026-10-14 en autonomie (`bmad-create-story`, Opus 5.5), sur
`16eeed21`. Contexte, retenu, écartées, réversibilité.*

---

## Story 29-1 — Préparer la clôture d'un exercice (#232)

### C-29-1-1 — Une fiche, deux stories (29-1a contrôles préalables, 29-1b transitoires) ; pas de 29-1c

- **Contexte** : l'ensemble touche plus de cinq modules (§ *Règle de splitting préventif*) ; le fichier d'epic
  prévoyait 29-1a / 29-1b / « éventuellement 29-1c affectation du résultat ».
- **Retenu** : précédents C-26-2-15 et C-28-4-1 — une fiche, deux clés, deux PR successives, clé mère `split`.
  29-1a (lecture seule, `refs #232`) puis 29-1b (contre-passation au début de l'exercice suivant, E2E de parcours,
  répétition à blanc au manuel d'administration, `closes #232`). Chacune compte cinq modules, la borne de la règle.
  Pas de 29-1c : l'affectation du résultat est écartée (C-29-1-2) ; la répétition à blanc va à la dernière
  sous-story, la 29-1b, comme le demande l'orchestrateur.
- **Écartées** : une story unique (trop large) ; trois stories avec une 29-1c « répétition à blanc » seule
  (documentation sans code, qui se relit avec la 29-1b dont elle décrit les gestes).
- **Réversible** : oui.

### C-29-1-2 — Ni écritures de clôture, ni écriture d'affectation du résultat

- **Contexte** : #232 demande la « génération des écritures de clôture (virement du résultat au bilan) » ;
  l'orchestrateur propose d'examiner une écriture d'affectation du résultat. Le modèle « temps réel virtuel »
  (Story 14-1, décision de Guy du 2026-07-21) calcule résultat de l'exercice et résultat reporté ; le compte 2979
  (`CurrentYearResult`) est non imputable.
- **Retenu** : rien n'est généré. Une écriture de virement doublerait le calcul et fausserait le bilan. Pour une
  personne physique ou un indépendant, l'affectation est sans objet (le résultat s'accumule aux fonds propres, ce que
  la ligne « Résultat reporté (calculé) » montre). Pour une société, l'affectation votée par l'assemblée reste une
  écriture **manuelle**, déjà possible (débit du compte physique de rôle `RetainedEarnings`, crédit dividendes ou
  réserve) ; sa **présentation** au bilan (compte physique négatif à côté de la ligne calculée positive) est
  perfectible — **demande d'issue** à l'orchestrateur, pour que la fermeture de #232 ne la fasse pas disparaître.
- **Écartées** : écriture d'affectation assistée (double un calcul, ou exige de revoir le modèle 14-1) ; retour à des
  écritures de clôture physiques (décision d'architecture contraire).
- **Réversible** : oui (une story future peut ajouter l'affectation assistée pour les sociétés).

### C-29-1-3 — Les contrôles signalent, ils ne bloquent pas ; `fiscal_years::close` n'est pas touchée

- **Contexte** : candidats de l'orchestrateur (contrôles en lecture seule, utilisables en cours d'année) ; leçon B5
  de la rétrospective 28 (toute simplification qui touche une transaction se relit sur les verrous et la naissance de
  la vue).
- **Retenu** : une lecture séparée, rappelée dans le dialogue de clôture, sans effet sur le `POST /close`. Le bouton
  *Clôturer* reste actif ; seul K1 (antérieur ouvert) bloque, et il bloquait déjà. Aucun verrou, aucune lecture ne
  s'ajoute à la clôture.
- **Écartées** : refuser la clôture tant qu'un contrôle est en `attention` (la clôture se défait par une réouverture
  motivée ; un refus côté serveur obligerait à lire les contrôles **sous** les verrous de la clôture — exactement le
  terrain de la 28-4) ; une case « j'ai vu les avertissements » envoyée au serveur (cérémonie sans garantie).
- **Réversible** : oui.

### C-29-1-4 — Sept contrôles, et ceux qui sont écartés

- **Retenu** : K1 antérieur ouvert, K2 transactions bancaires non rapprochées, K3 relevé de fin d'année (manquant,
  arrêté trop tôt, en écart), K4 factures en brouillon, K5 factures fournisseur importées à compléter, K6 écritures
  incohérentes héritées (déséquilibrées, moins de deux lignes, date hors de leur exercice, ou d'un autre exercice
  datées dans celui-ci), K7 exercice suivant absent.
- **Écartés**, un par un :
  - *comptes d'attente soldés* : Kesh n'en a pas (aucun rôle, drapeau ni compte ; inventaire au code) ; un rôle neuf
    serait une migration breaking (un binaire antérieur ne décode pas un rôle inconnu) — prémisse de #232 corrigée ;
  - *TVA réconciliée* : report assumé des P1 de la TVA (utilisateur non assujetti) ; demande d'issue ou de
    rattachement au jalon « Vague 2 » ;
  - *équation du bilan* : K6 en couvre les deux causes (écriture déséquilibrée ; ligne datée dans un exercice et
    rattachée à un autre — le compte de résultat filtre `fiscal_year_id`, le bilan non) ; un contrôle de l'équation
    répéterait K6 sans dire quelle écriture corriger ;
  - *verrou de période contre fin d'exercice* : la clôture fige tout l'exercice, le verrou n'ajoute rien à signaler ;
  - *postes ouverts débiteurs / créanciers* : une créance ouverte au 31 décembre est normale, l'écran *Postes
    ouverts* les liste déjà à une date ;
  - *factures fournisseur saisies ouvertes* : déjà comptabilisées (dette), rien d'anormal.
- **Réversible** : oui (un contrôle s'ajoute par un changement de la fiche et du test d'ensemble clos T-a1).

### C-29-1-5 — Lecture sous `ReadSnapshot`, tout rôle, sans audit

- **Contexte** : sept requêtes qui doivent voir la base au même instant ; les rapports lisent sur le pool sans
  instantané ; `ReadSnapshot` (Story 28-3) est conçu pour être réutilisé.
- **Retenu** : une connexion, une vue, `READ ONLY` ; route dans `authenticated_routes` (Consultation lit, comme les
  rapports) ; pas d'audit (lecture, pas un export).
- **Écartées** : lectures sur le pool (incohérence possible entre contrôles) ; audit `fiscal_year.checks_viewed`
  (bruit dans une piste de contrôle).
- **Réversible** : oui.

### C-29-1-6 — Le module `kesh-db/src/repositories/closing_checks.rs` et la route sous `/fiscal-years/{id}`

- **Retenu** : la lecture vit à côté des dépôts qu'elle interroge ; la route est une ressource de l'exercice.
- **Écartées** : `kesh-report` (états comptables, pas contrôles de données) ; `/reports/closing-checks?fiscalYearId=`
  (ce n'est pas un rapport exportable).
- **Réversible** : oui.

### C-29-1-7 — K3 réutilise la requête du tableau de bord, bornée, sans copie

- **Contexte** : seule `list_by_company_with_balances` sait corriger l'écart des décalages date de valeur /
  comptabilisation ; elle ne prend pas de date.
- **Retenu** : extraction d'une fonction privée au SQL **unique**, avec une borne facultative sur `period_to` ; le
  tableau de bord passe `None`, résultat inchangé (tests existants verts sans modification).
- **Écartées** : copier la requête (règle DRY ; deux copies divergeraient au prochain correctif) ; comparer seulement
  le dernier relevé sans borne (un relevé de janvier N+1 masquerait l'absence de celui de décembre).
- **Réversible** : oui.

### C-29-1-8 — Les transitoires : un geste « contre-passer au début de l'exercice suivant », sans marque ni migration

- **Contexte** : #232 demande « saisie des actifs/passifs de régularisation avec contre-passation automatique à
  l'ouverture de N+1 ». Kesh n'a pas d'événement « ouverture » (l'exercice suivant est créé par l'utilisateur, souvent
  avant la clôture de N) ; la contre-passation existe, datée du jour.
- **Retenu** : l'écriture de régularisation se saisit comme toute écriture ; un bouton de sa fiche la contre-passe au
  **premier jour de l'exercice qui suit le sien**, par la contre-passation unique (liaison `reverses_entry_id`,
  lettrage, refus, rejeu). « Automatique » s'entend : Kesh construit, date et lie l'inverse ; l'utilisateur déclenche.
- **Écartées** : (a) marque « transitoire » à la saisie + génération à la clôture de N — écrit sous les verrous de la
  clôture (cycle neuf avec les écrivains de N+1, terrain de la 28-4 et de #651), migration, et génération en masse qui
  décide seule ; (b) marque à la saisie + création de l'inverse dans la même transaction — l'inverse naîtrait avant
  que l'origine soit relue, et l'exercice suivant doit exister au moment de la saisie ; (c) contre-passation à date
  libre (C-29-1-9).
- **Réversible** : oui (une marque pourra s'ajouter plus tard, la machinerie restera).

### C-29-1-9 — Une seule date neuve : le premier jour de l'exercice suivant

- **Contexte** : le manuel fonde le verrou de période sur « la contre-passation est datée du jour : elle corrige
  depuis le présent ».
- **Retenu** : `ReversalDating { Today, NextFiscalYearStart }`. La date neuve tombe dans un exercice **postérieur** à
  celui de l'origine et passe par **tous** les refus de `create_in_tx_inner` (période verrouillée comprise).
- **Écartées** : une date choisie par l'utilisateur (rouvrirait la question du verrou de période, contre-passation
  datée dans le passé de l'origine).
- **Réversible** : oui.

### C-29-1-10 — Une route neuve, pas un corps sur `/reverse`

- **Retenu** : `POST /api/v1/journal-entries/{id}/reverse-at-next-fiscal-year-start`, sans corps.
- **Écartées** : un corps facultatif `{ "dating": … }` sur `/reverse` — les clients actuels envoient `{}` ou rien,
  et un extracteur JSON rendu obligatoire les casserait ; un paramètre de requête (un geste d'écriture distinct mérite
  une route nommée, inscrite au registre et rejouée).
- **Réversible** : oui.

### C-29-1-11 — L'exercice suivant : lecture ordinaire après `reversal_blockers`, confirmée par lecture verrouillante

- **Contexte** : B5 et § 4.2 de la rétrospective 28 — une lecture ordinaire avancée fait naître la vue plus tôt.
- **Retenu** : la vue naît déjà à `reversal_blockers` (première lecture ordinaire de la contre-passation) ; la lecture
  de l'exercice suivant se place **après**, au point où `today` est calculé ; `find_open_covering_date` (verrouillante,
  état validé le plus récent) confirme. Angle mort écrit au doc-comment : un exercice **intercalé** créé après la vue
  n'est pas vu.
- **Écartées** : une lecture verrouillante par intervalle pour trouver le suivant (ordre du parcours fixé par
  l'optimiseur, verrous de clé suivante — la doc de `fiscal_years::close` explique pourquoi la clôture y a renoncé).
- **Réversible** : oui.

### C-29-1-12 — Refus `NEXT_FISCAL_YEAR_MISSING` et `NEXT_FISCAL_YEAR_CLOSED`, en 409

- **Retenu** : deux codes distincts — l'un appelle à créer l'exercice, l'autre à le rouvrir ou à renoncer —, 409
  comme les autres refus d'état d'exercice (`EARLIER_FISCAL_YEAR_OPEN`).
- **Écartée** : réutiliser `FISCAL_YEAR_INVALID` (400, « aucun exercice ouvert ne couvre cette date ») — ne dit ni
  quelle date ni quoi faire.
- **Réversible** : oui (codes neufs, aucun client ne les attend encore).

### C-29-1-13 — E2E de parcours et répétition à blanc dans la 29-1b

- **Contexte** : Guy ne testera le bouclement qu'en début 2027 ; aucune recette en usage ne rattrapera ce que les
  revues manquent.
- **Retenu** : un E2E de parcours complet (contrôles, rapprochement, transitoire, contre-passation, bilans N et N+1,
  clôture, réouverture) ; au manuel d'administration, une marche de répétition sur une **instance séparée** à partir
  d'un `.keshbackup`, sans rien rapporter vers la production.
- **Réversible** : oui.

### C-29-1-14 — Angle mort assumé : une transitoire oubliée n'est pas repérée

- **Contexte** : sans marque (C-29-1-8), rien ne distingue une écriture de régularisation d'une autre.
- **Retenu** : écrit au manuel (AC17). Pas une dette au sens du `CLAUDE.md` (aucun défaut : une fonctionnalité non
  promise) ; si Guy la veut, CR à ouvrir.
- **Réversible** : oui.

### C-29-1-15 — Le défaut voisin « (si clôturé) » du manuel est corrigé dans la 29-1a

- **Contexte** : `user-manual.tex:728` affirme que les comptes du bilan sont reportés « depuis le bilan de clôture de
  l'exercice précédent (si clôturé) » ; au code, le report est calculé que l'exercice précédent soit clos ou non.
- **Retenu** : correction **mécanique** (une parenthèse retirée, une phrase juste), dans le commit du manuel de la
  29-1a (B3 : un défaut voisin mécanique s'intègre).
- **Réversible** : oui.
