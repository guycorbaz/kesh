# Epic 54 — Totaux du décompte de TVA

*Kickoff du 2026-11-21 (décision du Project Lead du 2026-11-21 : un total faux est un P1, il peut induire en erreur).*
Branche `epic/54-totaux-tva`, partie de `main` au lancement. Release visée : **après l'Epic 53**, au plus tard la release du
vendredi suivant sa clôture.

## Objet

Le décompte de TVA ne compte pas les avoirs : ses totaux sont faux, alors que le grand livre est juste (#390, #391, #392).
`crates/kesh-report/src/vat_report.rs` filtre les ventes sur `status = 'validated'`, et son contrôle de cohérence reprend
le même filtre, si bien qu'il rend « ok » même quand un avoir manque.

## Stories

| Story | Issues | Objet |
|---|---|---|
| 54-1 — Les avoirs dans le décompte de TVA | #390, #391, #392 | le décompte compte les avoirs (et les factures qu'ils annulent) ; le contrôle de cohérence se fait contre le grand livre, non contre le même filtre. La fiche juge s'il faut la découper. |
| 54-solde — Story de solde | plafond fixé au lancement | — |

## Dépendances

- Dépend de : aucune ; recoupe la 53-2 (comptes de TVA des avoirs) : les fiches s'accordent sur la source du compte de TVA.
- Bloque : aucune.
