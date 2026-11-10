# Epic 34 — Réconciliation et comptes bancaires

**Kickoff : 2026-11-10.** Guy a réinstallé Kesh à neuf sur le NAS le 2026-11-10 (0.17.5, livres réels). Les retours de
cette réinstallation et des premières réconciliations ont ouvert #966 à #972. Le périmètre a été proposé après la
rétrospective de l'Epic 33, et Guy l'a confirmé le 2026-11-10, en y ajoutant #970 et #972 (« oui, ajoute #972 et #970 »).

**Cible : v0.17.6.**

**Décomptes de départ** (§ *Priorités des défauts* du `CLAUDE.md`) :

- Au tag v0.17.5 : **261 issues ouvertes et 109 bugs** (rétrospective `epic-33-retro-2026-11-10.md`).
- Au kickoff : **265 issues, 109 bugs et 14 P1** (`gh issue list …`, 2026-11-10), 90 migrations.
- **Objectifs : moins de 261 issues et moins de 109 bugs au tag v0.17.6.**

⚠️ **Contrepartie BMAD des issues** : chaque issue tient le détail, et cet epic le périmètre. Il se clôt quand les issues
ci-dessous sont fermées et que la v0.17.6 est publiée.

## Règles de cet epic

- **Limites voisines** (décision de Guy du 2026-11-10, rétrospective de l'Epic 33, Q2) : la règle de l'Epic 33 est
  gardée, mais elle n'est pas absolue. Une story corrige elle-même les limites courtes qu'elle découvre, dans la mesure du
  possible. Ce qui n'est pas traité s'ouvre en issue, **une issue par problème, sans regroupement**.
- **Agents** : la consigne « l'outil Edit seulement » pour le code est maintenue (C5). Chaque écart se motive au
  Change Log.
- **Prompt commun** : il exige le nettoyage en fin de travail (conteneurs, dossiers `/tmp`, bases ; C2) et la recette
  E2E entière, avec ses six variables (C4).
- **Consignes des agents** : elles sont versées à `docs/agents-orchestration.md` dans la PR de release (C3).
- **Production** : aucun essai sur la base du NAS. Le dossier `~/kesh/` de la station contient les pièces de production
  de Guy. Il est en lecture seule : aucun agent n'y écrit, n'y lit de données pour un test ni ne les cite dans le dépôt.

## Stories

| Story | Issues | Objet | Migration |
|---|---|---|---|
| 34-1 | #966 | assistant de démarrage : description et compte comptable du compte bancaire, 1020 présélectionné s'il existe, « modifiable plus tard » écrit sur la page et au manuel | non |
| 34-2 | #533 (P2) | solde d'ouverture saisi à la création du compte bancaire, par le mécanisme du mode *Compléter* | non attendue |
| 34-3 | #967 (P2) | réconciliation : « Écriture manuelle » (l'ancien *Affecter manuellement*) et *Éclater* sur tout compte imputable sauf celui de la banque | non |
| 34-4 | #828 (P2) | réconciliation : signaler une transaction qui semble déjà comptabilisée, et confirmer avant de l'affecter | non attendue |
| 34-5 | #951 (P1) | règlement d'une facture : idempotence côté serveur | **probable** |
| 34-6 | #953 (P2) | édition de la société : le second essai après un 409 n'écrase plus l'écriture concurrente | non |
| 34-7 | #945 (P2) | renommer un exercice : rejeu sur interblocage | non |
| 34-8 | #938 (P2) | factures fournisseur : la relecture après commit n'annonce plus en échec un geste écrit | non |
| 34-9 | #969 (P2) | import du dossier : une ligne par fichier, doublon « déjà importé » avec lien vers la pièce existante, avertissement si un fichier échoue | non |
| 34-10 | #970 | accueil : barres en couleurs unies (recettes en vert, dépenses en rouge), anneau avec le total des dépenses au centre, parts en couleurs unies | non |
| 34-11a | #972 (P2) | carte de crédit : compte sans IBAN lié à un compte de dette, quatre derniers chiffres seulement ; inventaire clos des sites qui lisent l'IBAN | **probable** |
| 34-11b | #972 | carte de crédit : relevé (CSV ou saisie manuelle), réconciliation, prélèvement mensuel comme virement, manuel pas à pas | non attendue |

**Ordre et dépendances :**

- 34-11a et 34-11b viennent **après 34-3** : la contrepartie d'un achat par carte est souvent un compte de charge, mais le
  prélèvement passe par un compte de classe 2.
- 34-11b suit 34-11a. 34-2 touche le même formulaire que 34-1 et la suit.
- Toutes les autres stories sont indépendantes.
- Le découpage de #972 en deux stories applique la règle de splitting préventif : la story touche plus de cinq modules
  (comptes bancaires, import, réconciliation, QR-facture, `pain.001`, sauvegarde).
- Une story qui découvre qu'il lui faut une migration le dit avant de l'écrire.

## Décisions de Guy

**#966, 2026-11-10.** À l'assistant, le compte bancaire peut être lié, de manière facultative, à un compte du plan,
**présélectionné sur 1020 s'il existe**. La page et le manuel disent que ce choix est **modifiable après coup**. Aucun
rôle ne se déduit du numéro du compte (Story 14-3a) : 1020 n'est qu'une présélection.

**#967, 2026-11-10.** « Écriture manuelle » remplace « Affecter manuellement ». Tout compte imputable est proposé, sauf
celui de la banque : des intérêts et d'autres revenus doivent pouvoir s'y passer. Le filtre sur les classes 5, 6 et 7
est retiré dans les deux dialogues (*Écriture manuelle* et *Éclater*). La case à cocher par compte a été écartée.

**#968, 2026-11-10.** Les aides au survol et au focus forment un **epic à part**, hors de la 0.17.6.

**#969, 2026-11-10.** Le rapport liste **chaque fichier**, avec son statut. Un doublon n'est pas importé. Sa ligne dit
« déjà importé », renvoie à la pièce existante et dit que le fichier est passé dans `failed/`. Un avertissement apparaît
dès qu'un fichier échoue. Pas de migration.

**#970, 2026-11-10.** Les hachures sont retirées : les barres sont en couleurs unies, recettes en vert et dépenses en
rouge. Le camembert devient un **anneau**, avec le total des dépenses au centre. Ses parts sont en couleurs unies :
rouge, vert, bleu, puis deux ou trois autres si nécessaire.

**#972, 2026-11-10.** Une carte de crédit est tenue **comme un compte**, et non par « Éclater ». Le compte n'a pas
d'IBAN et il est lié à un compte de dette. Seuls les quatre derniers chiffres de la carte sont gardés : le numéro
complet n'est jamais enregistré. Le relevé entre par l'import CSV ou par la saisie manuelle. Le prélèvement mensuel est
un virement entre les deux comptes.

## Questions ouvertes, à trancher à la spécification

- **#972** : sous quelle forme Guy reçoit-il ses relevés de carte ? PDF seul, ou CSV/Excel ? Cela décide si la 34-11b
  suffit telle quelle.
- **#951** : la protection contre le double règlement passe-t-elle par une version ou par une clé d'idempotence ? Dans
  les deux cas, le contrat de la route change.

## Production

**Deux migrations probables** (34-5 et 34-11a). Si l'une d'elles est écrite :

- la sauvegarde d'avant la mise à jour s'écrit en tête des notes de version et au manuel d'administration ;
- Kesh écrit en plus sa propre sauvegarde dans `backup/pre-upgrade/` (#847) ;
- la montée se répète d'abord sur une base peuplée par l'image 0.17.5 publiée (`scripts/recette-montee.sh`).
