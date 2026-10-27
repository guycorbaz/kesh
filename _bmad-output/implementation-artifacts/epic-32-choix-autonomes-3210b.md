# Epic 32 — choix autonomes de la Story 32-10b (l'écran des prompts du serveur MCP)

*Registre ouvert le 2026-10-27 à la rédaction de la fiche `32-10b-ecran-prompts.md` (Opus 5.5, en autonomie, contre
`42680725`). Chaque choix : contexte, option retenue, alternatives écartées, réversibilité. Ce qui revient à Guy est au
§ *Points à soumettre à Guy* de la fiche, ce qui revient à l'orchestrateur au § *Demandes à l'orchestrateur*. À verser
par union au registre de l'epic à la clôture.*

### C-32-10b-1 — Le texte servi est calculé par le serveur, non recomposé par l'écran

- **Contexte** : pour une version d'agent, le texte servi est la marque, une ligne vide, puis le texte (AC-ii7 de la
  32-10a) ; la route `GET /{id}` de la fiche 32-10a rend « le prompt et tout son historique, textes compris », c'est-à-
  dire le texte **stocké**. Montrer le texte stocké, c'est faire approuver autre chose que ce qui part.
- **Retenu** : demander à l'orchestrateur (D1) que la route rende, par version de nature texte, `servi: {description,
  texte, marque}` calculé par la fonction du catalogue — un seul code compose ce que l'agent reçoit ; l'écran l'affiche.
  Variante B écrite (recomposition côté écran, gabarit gardé contre la source Rust) si l'amendement n'est pas fait.
- **Écartées** : recomposer côté écran d'emblée (deux codes pour une phrase ; un écart ferait approuver un texte non
  servi tel quel) ; une route « aperçu » neuve (écart à « la 32-10b n'ajoute aucune route », et elle toucherait `routes`
  et l'audit) ; montrer le texte stocké avec la marque en légende (la marque n'y serait pas à sa place).
- **Réversible** : oui, tant que la 32-10a-iii n'est pas développée ; ensuite, la variante B reste possible.
- **Annoté à la validation P1 (2026-10-27)** : D1 accepté, puis **étendu** par l'orchestrateur (fiche 32-10a, `e7a90572`) — `servable`, champs rendus visibles par le serveur, texte du code d'un standard, noms et date. La **variante B est retirée** : sans l'amendement, le développement s'arrête et rend compte ; elle apportait son propre angle mort (une composition serveur modifiée sans toucher au gabarit).
- **Annoté à la validation P2 (2026-10-27)** : la note complétée de la 32-10a (`20a02a86`) rend le texte et la description servis, chacun exact et rendu visible ; **la marque fait partie du texte servi, il n'y a pas de champ `marque` séparé** — le `servi: {description, texte, marque}` du « Retenu » est périmé. La composition vit dans `services/` (garde `un_seul_point_d_entree` de la 32-6).

### C-32-10b-2 — La marque dans le texte, encadrée, à sa place

- **Contexte** : « la marque visible d'un prompt d'agent, à l'écran comme dans le texte servi ».
- **Retenu** : la marque est montrée **dans** le bloc du texte servi, en tête, encadrée et précédée d'une pastille
  *Écrit par un agent* ; si le texte servi ne commence pas par la marque attendue, il est montré tel quel, sans encadré —
  jamais masqué. La pastille *Agent* figure aussi dans la liste et l'historique.
- **Écartées** : une pastille seule (l'Administrateur ne verrait pas ce que l'agent lit) ; masquer la marque du texte
  pour « alléger » (contraire à la propriété).
- **Réversible** : oui.
- **Annoté à la validation P2 (2026-10-27)** : l'**encadré séparé est retiré** (R1, F6 ; consigne « retirer plutôt qu'enrichir ») : la marque se lit dans le texte servi rendu visible, à sa place, sans traitement de l'écran — ni encadré, ni comparaison au début du texte, ni avis « marque manquante ». La pastille *Écrit par un agent*, tirée de l'origine de la version, reste hors du `<pre>`. La preuve : l'égalité du `textContent` avec le champ visible sur une version d'agent.
- **Annoté à la validation P3 (2026-10-27)** (F7, R2) : le titre (« encadrée ») et le « Retenu » ci-dessus sont **périmés** depuis la P2 ; ce qui vaut est l'annotation P2. Ils restent écrits comme trace du choix d'origine.

### C-32-10b-3 — Un signalement des caractères cachés côté écran, miroir gardé de `kesh-core`

- **Contexte** : C-32-8-20 a écarté une copie TypeScript du prédicat pour l'écran des envois, en la disant réversible
  (« un signalement côté client peut s'ajouter, à titre d'aide »). Pour un prompt, le serveur refuse ces caractères à
  l'écriture, mais une version restaurée ou écrite hors de Kesh peut en porter, et la saisie de l'Administrateur aussi.
- **Retenu** : `segmenter` signale les classes `Cc` (selon le mode), `Cf`, `Co`, `Cn` du moteur JavaScript et les deux
  listes écrites de `is_hidden_format`, recopiées et **tenues égales** à la source Rust par une garde ; une seconde
  garde exige que Node rende la même version d'Unicode que la crate (17.0). Rendu en pastilles `⟦U+XXXX⟧`,
  `reconstituer(segmenter(t)) === t`.
- **Écartées** : aucun signalement (un caractère invisible ne se voit pas, c'est sa définition) ; un signalement calculé
  par le serveur (une route ou un champ de plus, hors des trois modules) ; insérer le caractère lui-même avec un style
  (il resterait invisible).
- **Réversible** : oui.
- **Remplacé à la validation P1 (2026-10-27)**, décision de l'orchestrateur « fermer les classes par la conception, pas par des gardes » (R1, R2 HIGH ; F1 HIGH ; E13, E14) : **l'écran n'a plus aucune logique Unicode**. Le serveur rend visibles les caractères cachés (`rendre_visible`, 32-8b) et rend le verdict `servable` ; l'écran affiche le champ visible sans transformation (test au niveau du DOM). Retirés : `segmenter`, `reconstituer`, la copie des listes de `kesh-core` et sa garde, la garde de version d'Unicode du moteur, les pastilles calculées dans le navigateur. Motif : la garde de version était irréalisable dans le job Frontend de la CI (aucun fichier Rust, Node 22), et le mode « ligne » signalait à tort le `\n\n` de toute description d'agent.

### C-32-10b-4 — Signaler, ne pas bloquer ; montrer aussi ce que le serveur admet

- **Contexte** : les tables Unicode du navigateur peuvent différer de celles du serveur ; l'angle mort de `kesh-core`
  admet des espaces `Zs`, U+2800, U+1D159, U+16FE4.
- **Retenu** : deux comptes distincts — `refuse` (avertissement) et `admis` (information : `\t` en mode texte, `Zs`
  autres que U+0020, les trois points de l'angle mort) ; **aucun bouton désactivé** par le signalement ; le serveur
  reste seul juge. D2 demande que le serveur refuse l'approbation d'une version invalide.
- **Écartées** : désactiver *Approuver* sur un signalement `refuse` (un navigateur plus ancien bloquerait un texte
  légitime à caractère récent) ; ne signaler que ce que le serveur refuse (l'Administrateur ne verrait pas une espace
  fine insécable qui part à l'agent).
- **Réversible** : oui.
- **Remplacé à la validation P1 (2026-10-27)** : plus de comptes `refuse` et `admis`. Le verdict est celui du serveur (`servable`) ; *Approuver* est absent sur une version non servable (C-32-10b-17), le serveur refusant aussi (D2, accepté). Les caractères **admis** ne sont plus montrés (le rendu du serveur ne les vise pas) : angle mort écrit, point 3 pour Guy reformulé.

### C-32-10b-5 — On approuve la version lue, sans relecture avant le `POST`

- **Contexte** : le risque principal de la story ; patron de la décision « tel quel » de la 32-8c (version lue,
  409 si périmée).
- **Retenu** : l'approbation et le refus envoient le `numero` et la `version` **de l'objet affiché** ; aucun `GET`
  entre l'affichage et le `POST` ; la relecture vient après la décision. Une proposition arrivée entre-temps rend la
  demande périmée (409 du serveur, AC-i7 de la 32-10a) et l'écran propose *Recharger*, sans relance automatique.
- **Écartées** : relire le détail juste avant d'envoyer (on approuverait la version du serveur, non celle lue) ;
  relancer automatiquement après un 409.
- **Réversible** : non souhaitable.

### C-32-10b-6 — *Approuver* sur la dernière version seulement, derrière un dialogue qui restate

- **Contexte** : AC-i7 de la 32-10a (seule la dernière version `agent` `pending` s'approuve) ; approbation par mégarde.
- **Retenu** : le bouton n'est rendu que sur cette version ; une version à valider plus ancienne n'a que *Refuser*,
  avec la raison ; *Approuver* ouvre un dialogue qui restate nom, numéro, clé auteur, la portée qui la servira et la
  marque. La version à valider est montrée en entier, jamais repliée.
- **Écartées** : le bouton partout, le serveur répondant 409 (geste offert puis refusé : confusion) ; une case « J'ai
  lu » (question 5 pour Guy : un réflexe de plus) ; une approbation d'un clic.
- **Réversible** : oui.

### C-32-10b-7 — La version en vigueur : du serveur, sinon par la règle, en fonction pure

- **Contexte** : la synthèse calcule la version en vigueur (AC-i5) ; la fiche 32-10a ne dit pas si le détail la rend.
- **Retenu** : le champ du serveur s'il existe (demandé par D1) ; sinon, la règle de l'AC-i6 (approuvée de plus grand
  numéro, de toute nature) en fonction pure testée sur les cas `pending`, `refused` et retrait.
- **Écartée** : toujours recalculer côté écran (une seconde implémentation d'une règle du serveur).
- **Réversible** : oui.
- **Annoté à la validation P2 (2026-10-27)** : le repli « sinon, par la règle » est **retiré** (R5) : `numeroEnVigueur` est rendu par le serveur (note `20a02a86`) et figure à la liste d'arrêt de l'étape 0 ; l'écran ne recalcule rien.
- **Annoté à la validation P3 (2026-10-27)** (F7) : le titre (« sinon par la règle, en fonction pure ») et le « Retenu » ci-dessus sont **périmés** depuis la P2 ; ce qui vaut est l'annotation P2.

### C-32-10b-8 — L'éditeur : valeurs exactes, aperçu des caractères cachés, avis P26, retrait

- **Contexte** : C-32-10a-19 (« l'écran pré-remplit, l'administrateur enregistre ») ; P26 de la 32-10a (un texte
  d'agent enregistré comme version d'administrateur perd sa marque, « écrit au manuel par la 32-10b »).
- **Retenu** : description et texte envoyés exacts (fabriques de corps seules) ; compteurs sans blocage ; aperçu des
  caractères cachés de la saisie ; un avis quand l'éditeur est pré-rempli depuis une version d'agent ; *Retirer* derrière
  un dialogue.
- **Écartées** : bloquer côté écran les longueurs ou les caractères (C-32-8-20 : le serveur juge) ; interdire de
  reprendre un texte d'agent (l'Administrateur doit pouvoir corriger une proposition) ; garder la marque sur une version
  d'administrateur (la marque dit qui a écrit, et c'est l'Administrateur).
- **Réversible** : oui.
- **Annoté à la validation P1 (2026-10-27)** : l'aperçu des caractères cachés de la saisie et les compteurs « N / 16 000 », « N / 500 » sont **retirés** (logique Unicode côté écran ; R13 tombe) ; une aide dit les bornes, le serveur refuse (400, saisies conservées).
- **Annoté à la validation P4 (2026-10-27)** (H3, H6) : après toute relecture réussie (décision, geste, *Recharger*), l'éditeur est réinitialisé sur la version en vigueur (P3, F5) ; la saisie non enregistrée est perdue, et l'AC-12 et le manuel le disent ; il n'est jamais pré-rempli par le texte servi (P3, R1).

### C-32-10b-9 — Pas de `+page.ts` : le 403 de l'API, affiché

- **Contexte** : la 32-8c n'a pas de garde de page (fait 2) ; la 32-7 en prévoit une (`admin/backup/+page.ts`).
- **Retenu** : le motif de l'écran voisin fusionné, la 32-8c ; l'entrée de menu est `adminOnly`, le serveur est
  l'autorité.
- **Écartée** : une redirection de page (une seconde forme pour deux écrans voisins ; à harmoniser, si Guy le souhaite,
  avec la 32-7, pour les trois écrans d'un coup).
- **Réversible** : oui.

### C-32-10b-10 — *Revenir au texte standard* quand c'est utile, et ce que le geste fait de l'historique

- **Contexte** : « le retour au standard et ce qu'il fait de l'historique : jamais d'écrasement » ; AC-iii11 (d) de la
  32-10a (le prompt redevient non personnalisé).
- **Retenu** : bouton sur un prompt standard personnalisé, retiré par l'Administrateur, à version d'agent en vigueur, ou
  au standard changé ; le dialogue dit qu'une version de plus est créée, que l'historique reste, et que le prompt suivra
  de nouveau les versions de Kesh.
- **Écartées** : montrer le texte standard du code avant le geste (la route ne le rend pas ; demande de plus à la
  32-10a pour un gain faible : il paraît en tête de l'historique aussitôt après) ; le bouton toujours présent.
- **Réversible** : oui.
- **Annoté à la validation P1 (2026-10-27)** (R7, F2) : l'alternative écartée « montrer le texte standard du code avant le geste » est **retenue** — D1 étendu rend `standardDuCode`, rendu visible, que le dialogue montre avant la confirmation ; inventaire E16. La condition du bouton devient le seul drapeau `standardDiffere` du serveur, qui couvre déjà les cas énumérés (R3).
- **Annoté à la validation P2 (2026-10-27)** : la condition devient `standardDiffere && standardDuCode !== null` (F5, R11 : un standard que le binaire ne porte plus a le drapeau levé, aucun texte du code, et la route le refuse) ; le cas « version d'agent approuvée » sur un standard est retiré, inatteignable (C-32-10a-27).
- **Annoté à la validation P3 (2026-10-27)** (F7) : l'énumération du « Retenu » (« à version d'agent en vigueur, ou au standard changé ») est **périmée** ; la condition est celle de l'annotation P2.

### C-32-10b-11 — Le repli égal au catalogue : G13 par préfixe si la 32-7 l'a posé, sinon un module partagé

- **Contexte** : #758 ; la 32-8c a fermé la classe pour son préfixe par une garde locale (C-32-8-71) ; la 32-7 étend G13
  par préfixe (C-32-7-21), non fusionnée.
- **Retenu** : (a) ajouter le préfixe à `PREFIXES_REPLI_EGAL_AU_CATALOGUE` si elle existe ; (b) sinon, sortir le relevé et
  `ecartsAuCatalogue` de la garde des envois en un module partagé, employé par les deux gardes — DRY.
- **Écartées** : recopier la garde de la 32-8c (deux copies d'une même fonction) ; ne rien garder (angle mort de #758
  élargi).
- **Réversible** : oui.

### C-32-10b-12 — La sous-section du manuel après `sec:mcp-envois`, sous *Sécurité* provisoirement

- **Contexte** : la 32-8c a placé `sec:mcp-envois` sous *Sécurité* faute de `sec:mcp` ; la 32-7 rangera les
  sous-sections sous sa section.
- **Retenu** : `\subsection{Les prompts du serveur MCP}\label{sec:mcp-prompts}` après `sec:mcp-envois`, sous `sec:mcp`
  si elle existe à l'étape 0, sinon sous *Sécurité* ; la 32-7 l'y rangera (demande D3).
- **Écartée** : une section de premier niveau (elle devrait être déplacée deux fois).
- **Réversible** : oui (étiquette stable).

### C-32-10b-13 — Les chemins : `/admin/mcp/prompts` et `features/mcp-prompts`

- **Contexte** : C-32-8-21 (`/admin/mcp/sends`, `features/mcp-sends`) ; la 32-7 occupe `/admin/mcp` ; les routes de
  l'API sont sous `/api/v1/admin/mcp/prompts` (32-10a).
- **Retenu** : écran `/admin/mcp/prompts` et `/admin/mcp/prompts/[id]`, feature `mcp-prompts`, entrée de menu propre
  `nav-admin-mcp-prompts` après celle des envois.
- **Écartée** : un onglet de l'écran *Serveur MCP* de la 32-7 (dépendance d'ordre entre stories non fusionnées).
- **Réversible** : oui.

### C-32-10b-14 — Une E2E au-delà du minimum de l'epic

- **Contexte** : l'epic demande « E2E approbation et refus » ; la consigne ajoute « la valeur servie par `prompts/get`
  égale à celle qui a été vue ».
- **Retenu** : (a) approbation, valeur vue = valeur servie, marque comprise ; (b) refus ; (c) nouvelle version
  d'administrateur reprise d'une version d'agent (avis P26, servie sans marque) ; (d) retour au standard ; (e) un
  Comptable. Chaque cas compare au `prompts/get` réel.
- **Écartée** : les seuls (a) et (b) : (c) et (d) sont les gestes de rattrapage d'une approbation par mégarde, que seul
  un E2E voit traverser la frontière HTTP.
- **Réversible** : oui.

### C-32-10b-15 — Pas de comparaison avec la version en vigueur en 0.16.0

- **Contexte** : un agent pourrait glisser une petite modification dans un long texte.
- **Retenu** : le texte entier, jamais replié, et un dialogue qui restate ; la comparaison est la question 4 pour Guy,
  en issue si la recette la demande (D4).
- **Écartée** : un composant de différence dès cette story (une dépendance ou un algorithme de plus, hors du risque
  « faible à moyen » du kickoff).
- **Réversible** : oui.

### C-32-10b-16 — La création d'un prompt sur la page de la liste

- **Contexte** : la route `POST /api/v1/admin/mcp/prompts` existe (AC-iii6) ; le kickoff ne nomme pas la création.
- **Retenu** : un formulaire *Nouveau prompt* sur la page de la liste, puis navigation vers le détail créé ; la règle du
  nom est dite, non contrôlée côté écran.
- **Écartées** : une page `/admin/mcp/prompts/nouveau` (une route de plus pour trois champs) ; ne pas offrir la création
  (la route existerait sans écran : l'Administrateur ne pourrait écrire qu'en modifiant un standard).
- **Réversible** : oui.

### C-32-10b-17 — *Approuver* absent sur une version que le serveur ne servirait pas

- **Contexte** : validation P1 (F3, E14) ; D2 accepté (le serveur refuse d'approuver une version non servable) ;
  AC-14 (l'écran n'offre aucun geste que la route refuse).
- **Retenu** : sur une version d'agent à valider dont `servable` (verdict du serveur, D1 étendu) est faux, pas de
  bouton *Approuver* ; l'avis de l'AC-6 et *Refuser* seuls. Le bloc « Ce que l'agent reçoit aujourd'hui » n'est montré
  que pour une version en vigueur servable.
- **Écartées** : le bouton présent, le serveur répondant 409 (geste offert puis refusé) ; un blocage calculé par
  l'écran (logique Unicode côté client, retirée).
- **Réversible** : oui.
- **Annoté à la validation P2 (2026-10-27)** : le refus du serveur est un **400**, « le même code que la création » (D2, note `20a02a86` ; R4), non un 409 ; l'AC-12 le range sous le 400.
- **Annoté à la validation P3 (2026-10-27)** (F6) : le bloc « Ce que l'agent reçoit aujourd'hui » s'appelle désormais *Texte servi par Kesh* (il ne suppose pas le serveur MCP activé).

### C-32-10b-18 — Un seul envoi à la fois sur chaque geste

- **Contexte** : validation P1, F8 (double clic : deux `POST`, une erreur affichée par-dessus un succès).
- **Retenu** : chaque bouton qui envoie est désactivé (`disabled`, `aria-busy`) de la confirmation à la réponse ; la
  réponse remplace l'état ; test paramétré par geste, mutation M17.
- **Écartée** : s'en remettre au 409 du serveur (le second refus s'afficherait sur un geste réussi).
- **Réversible** : oui.
- **Annoté à la validation P2 (2026-10-27)** : **un seul** état d'envoi pour **tous** les gestes de la page, comme la 32-8c (`McpSendDetail.svelte:47`) — F8 ; le test `gestes_desactives_pendant_l_envoi` asserte l'état `disabled` de chaque geste (F3, R13 : compter les appels ne tuait pas M17) ; le retrait du retour anticipé du gestionnaire est déclaré mutation équivalente, non jouée.
- **Annoté à la validation P3 (2026-10-27)** (R2, R5, F4, F10) : « la réponse remplace l'état », au « Retenu », est **retiré** depuis la P2 (non testé) ; la mutation du retour anticipé **ne l'est pas** (voir l'annotation P2) — Svelte appelle le gestionnaire délégué même sur un bouton désactivé — : elle devient **M18**, tuée par deux `click` envoyés par `fireEvent` et un seul `POST` attendu. Pendant l'envoi, les dialogues ne se ferment pas (les trois réglages de `McpSendDetail.svelte:345-347`).

### C-32-10b-19 — Le drapeau `standardDiffere` n'est pas affiché tel quel

- **Contexte** : validation P1, R3 — le drapeau (AC-i5 de la 32-10a) est vrai pour tout standard personnalisé ;
  « Le standard a changé » aurait été affiché sur chaque prompt personnalisé.
- **Retenu** : *Personnalisé* quand la version en vigueur d'un standard n'est pas d'origine `standard` ; « Diffère du
  standard de cette version de Kesh » seulement pour une version d'origine `standard` au drapeau levé ; le drapeau seul
  conditionne *Revenir au texte standard*.
- **Écartées** : afficher le drapeau sous un libellé unique (deux situations confondues) ; demander à la 32-10a un
  second drapeau (inutile : l'origine est déjà dans la synthèse).
- **Réversible** : oui.
- **Annoté à la validation P3 (2026-10-27)** (R2, F7) : « le drapeau seul conditionne *Revenir au texte standard* » est **périmé** depuis la P2 ; la condition est `standardDiffere && standardDuCode !== null` (C-32-10b-10, annotation P2).
