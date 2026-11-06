# Epic 33 — La numérotation interne des factures fournisseur, les graphiques de l'accueil, et moins de défauts

**Kickoff : 2026-11-06.** La v0.17.4 est en fonction sur le NAS, avec les livres réels de Guy. Il demande pour la
prochaine release #926 (« critique : la numérotation actuelle peut induire en erreur »), #922, #918 « et quelques bugs de
ton choix », dans un epic fait de petites stories, le plus possible indépendantes. Menée **en autonomie** jusqu'à l'image
Docker (« oui, de manière autonome »).

**Cible : v0.17.5** (décision de Guy du 2026-11-06). ⚠️ Cet Epic 33 n'est plus celui qu'annonçait la rétrospective 32
(propositions de l'agent, #235) : ceux-ci prendront le numéro suivant. La 0.18 reste à l'extraction fiscale.

**Décomptes de départ** (§ *Priorités des défauts* du `CLAUDE.md`) : **246 issues ouvertes et 91 bugs au tag v0.17.4**
(rétrospective `release-0.17.4-retro-2026-11-06.md`, § 2), 13 P1 au tag, 15 aujourd'hui (Guy a classé #926 et #922 en P1 le 2026-11-06), 89 migrations. **Objectifs : moins de 246 issues, moins
de 91 bugs au tag v0.17.5.**

⚠️ **Contrepartie BMAD des issues** — chaque issue tient le détail, cet epic le périmètre. Il se clôt quand les issues
ci-dessous sont fermées et la v0.17.5 publiée.

## Règle de cet epic pour les limites voisines (décision de Guy du 2026-11-06)

**Une story corrige elle-même les limites courtes qu'elle découvre, et ne trace en issue que ce qui déborde.** « On trace
toujours » reste entier pour ce qui n'est pas traité ; aucun défaut réel ne se ferme pour le chiffre. Motif : sur la 0.17.4,
treize des quatorze bugs ouverts venaient des fiches et revues de la release elle-même.

## Stories

| Story | Issues | Objet | Migration |
|---|---|---|---|
| 33-1a | #926 (P1) | numéro interne : colonne, format réglable, attribution à la validation, numérotation des factures existantes | **oui, la seule de l'epic** |
| 33-1b | #926 | le numéro à l'écran : liste, fiche, réglage, recherche, manuels | non |
| 33-2 | #922 (P1), #915 | import CAMT : refus si la relecture héritée échoue, faux doublon de l'alias | non |
| 33-3a | #918 | agrégats mensuels et postes de charges, par exercice ouvert | non |
| 33-3b | #918 | les deux graphiques SVG de la page d'accueil, clic vers le détail | non |
| 33-4 | #606, #742, #755 (P2) | bornes d'années des dates reçues du client et de l'expiration des clés | non |
| 33-5 | #909 (P2) | corriger l'adresse de la société après l'assistant | non |
| 33-6 | #450 (P2) | le règlement n'est plus annoncé en échec après son commit | non |
| 33-7 | #435, #729, #776 (P2) | journal d'audit : changement de mot de passe, export, consultation par le MCP | non |
| 33-8 | #486 (P2) | un compte désigné dans les paramètres ne s'archive plus sans avertissement | non |
| 33-9 | #424, #287 | le test E2E `sidebar-navigation` cesse de lancer de fausses alertes | non |
| 33-10 | #925 ; B1, B3 de la rétrospective 0.17.4 (B2 est dans `prompts/e33-commun.md`) | gate de release en mode CI, script de montée et de retour au dépôt | non |

33-1b suit 33-1a, 33-3b suit 33-3a ; toutes les autres sont indépendantes. Une story qui découvrirait qu'il lui faut une
migration le dit avant de l'écrire (numérotation).

## Décisions de Guy

**#926, 2026-11-06.** Le numéro du fournisseur reste, facultatif : il sert aux recherches, mais deux fournisseurs peuvent
avoir la même numérotation, et certains n'en ont pas. Kesh ajoute un **numéro interne unique et auto-généré**, pour classer
et retrouver les **factures papier**.

- Attribution **à la validation** : un brouillon n'a pas de numéro ; une facture repassée en brouillon garde le sien.
- Année de l'exercice, séquence qui repart à 1 à chaque exercice, « dans l'idéal configurable ».
- Format réglable comme pour les factures clientes ; **défaut `FF-{YEAR}-{SEQ:03}`**, pour distinguer les deux familles.
- **Les factures déjà saisies et validées sont numérotées** à la migration, par ordre de date, sans réécrire le libellé de
  leurs écritures passées. Les nouvelles écritures portent le numéro.
- **Visible** : liste (colonne triable et cherchable), fiche, libellé des écritures d'achat et de règlement. La pièce reçue
  reste intacte.

**#922, 2026-11-06.** Si la relecture « à la 0.17.3 » échoue, Kesh **refuse l'import de ce relevé** avec un message clair.

**#918, 2026-11-05.** À partir de la comptabilité (produits et charges passés au journal) ; exercice ouvert, même vide ;
barres mensuelles revenus / dépenses sur douze mois, sans ligne de résultat ; postes **par compte**, **camembert à cinq
parts plus « Autres »** ; clic vers le détail ; écritures de clôture et contre-passations écartées ; dessin en SVG, sans
bibliothèque.

## Production

**33-1a est la première migration à passer par la sauvegarde automatique (#847) sur le NAS.** Le CHANGELOG et les notes de
version recommandent aussi une sauvegarde manuelle avant la mise à jour, et disent que le retour à la 0.17.4 passe par elle.
