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

### C-32-10b-2 — La marque dans le texte, encadrée, à sa place

- **Contexte** : « la marque visible d'un prompt d'agent, à l'écran comme dans le texte servi ».
- **Retenu** : la marque est montrée **dans** le bloc du texte servi, en tête, encadrée et précédée d'une pastille
  *Écrit par un agent* ; si le texte servi ne commence pas par la marque attendue, il est montré tel quel, sans encadré —
  jamais masqué. La pastille *Agent* figure aussi dans la liste et l'historique.
- **Écartées** : une pastille seule (l'Administrateur ne verrait pas ce que l'agent lit) ; masquer la marque du texte
  pour « alléger » (contraire à la propriété).
- **Réversible** : oui.

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
