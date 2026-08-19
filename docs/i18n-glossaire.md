# Glossaire de traduction — fr-CH → de-CH / it-CH / en-CH

*Établi le 2026-08-19 au kickoff de l'**Epic 23 « Dette i18n »** (issues [#316] et [#283]).*

Ce glossaire existe parce que l'epic 23 fait entrer **307 clés** au catalogue et écrit
**921 messages** en trois langues. Sans terminologie figée d'avance, trois cents décisions
lexicales se prennent une par une, au fil des stories de rollout, et le produit finit par
dire deux mots différents pour la même chose selon l'écran.

⚠️ **Les équivalences de la partie A ne sont pas des propositions : elles sont RELEVÉES
dans les catalogues existants**, sur les **1216 clés déjà alignées sur les quatre locales.
La colonne « précédent » nomme la clé où l'équivalence est attestée.** Les changer, c'est
désaligner le nouveau du déjà-livré — ce que cet epic vient précisément corriger.

La partie B, elle, appelle un arbitrage : **ces termes n'ont AUCUN précédent** dans les
catalogues. Ce qui y est écrit est une proposition, pas un relevé.

---

## Registre — mesuré, pas supposé

| Locale | Registre | Mesure |
|---|---|---|
| `fr-CH` | vouvoiement | 44 messages avec « vous / votre » |
| `de-CH` | **Sie-Form** | 115 messages avec « Sie » |
| `it-CH` | **2ᵉ personne du singulier** (« Configura », « Scegli ») | 31 contre **1** au pluriel de courtoisie |
| `en-CH` | impératif neutre, sans pronom | — |

⚠️ **L'italien tutoie et l'allemand vouvoie.** Ce n'est pas une incohérence à corriger :
c'est l'usage courant des interfaces dans ces deux langues. La seule vraie anomalie est
`Aggiungete un conto al piano dei conti.` — **unique** occurrence du pluriel de courtoisie
en italien, à aligner au passage du rollout qui touchera ce domaine.

---

## A. Termes attestés — équivalences relevées dans le catalogue

| fr-CH | de-CH | it-CH | en-CH | précédent |
|---|---|---|---|---|
| facture | Rechnung | fattura | invoice | `Factures` / `Rechnungen` |
| facture fournisseur | Lieferantenrechnung | fattura fornitori | supplier invoice | `Factures fournisseurs` |
| avoir | **Gutschrift** | **nota di credito** | credit note | `Avoir` / `N° d'avoir` |
| brouillon | Entwurf | bozza | draft | `Brouillon` |
| ouverte (facture) | offen | aperta | open | `Factures ouvertes` |
| payée / impayée | bezahlt / unbezahlt | pagata / non pagata | paid / unpaid | `Payées` / `Impayées` |
| en retard | **überfällig** | in ritardo | overdue | `En retard` |
| échéance | **Fälligkeit** | scadenza | due date | `Échéance` |
| conditions de paiement | Zahlungsbedingungen | condizioni di pagamento | payment terms | `Conditions de paiement` |
| écriture (comptable) | **Buchung** / Buchungssatz | registrazione / scrittura contabile | entry / journal entry | `Dernières écritures` |
| compte | Konto | conto | account | `Comptes bancaires` |
| compte bancaire | Bankkonto | conto bancario | bank account | `Comptes bancaires` |
| solde | Saldo | saldo | balance | `Soldes de départ` / `Anfangssaldi` |
| montant | **Betrag** | **importo** | amount | `Montant trop élevé` |
| montant HT | Betrag exkl. MWST | importo IVA esclusa | amount excl. VAT | `Montant HT` |
| exercice (comptable) | **Geschäftsjahr** | esercizio contabile | fiscal year | `Exercices comptables` |
| résultat de l'exercice | Jahresergebnis | risultato dell'esercizio | current year result | `Résultat de l'exercice` |
| bilan | Bilanz | bilancio | balance sheet | `Bilan` |
| compte de résultat | **Erfolgsrechnung** | **conto economico** | income statement | `Compte de résultat` |
| journal | Journal | **giornale** | journal | `Journal` |
| TVA | **MWST** | IVA | VAT | `Taux TVA` / `MWST-Satz` |
| taux (TVA) | Satz | aliquota | rate | `Taux TVA` |
| décompte TVA | MWST-Abrechnung | rendiconto IVA | VAT settlement | `Décompte TVA` |
| TVA due | geschuldete MWST | IVA dovuta | VAT payable | `TVA due` |
| produit (compte de) | **Ertrag** | **ricavo** | revenue | `Produit` / `Produit par défaut` |
| créances clients | Forderungen aus Lieferungen und Leistungen | crediti verso clienti | trade receivables | `Créances clients` |
| dettes fournisseurs | Verbindlichkeiten aus Lieferungen und Leistungen | debiti verso fornitori | trade payables | `Dettes fournisseurs` |
| client | Kunde | cliente | client | `Client` |
| numéro de client | Kundennummer | numero cliente | client number | `Numéro de client` |
| contact | Kontakt | contatto | contact | `Nouveau contact` |
| carnet d'adresses | **Kontakte** | **contatti** | contacts | `Carnet d'adresses` |
| adresse | Adresse | indirizzo | address | `Adresse` |
| rappel / relance | **Mahnung** | **sollecito** | reminder | `Rappels` / `Mahnungen` |
| paiement | Zahlung | pagamento | payment | `Paiements fournisseurs` |
| réconciliation | **Abstimmung** (manuelle : Abgleich) | riconciliazione | reconciliation | `Réconciliation` |
| import bancaire | Bankimport | importazione bancaria | bank import | `Import bancaire CAMT.053` |
| sauvegarde | Sicherung | backup | backup | `Sauvegarde complète` |
| archiver / archivé | archivieren / archiviert | archiviare / archiviato | archive / archived | `Archiver` / `Archivé` |
| invalide | ungültig | non valido | invalid | `Valeur invalide` |
| référence | Referenz | riferimento | reference | `Référence` |
| utilisateur | Benutzer | utente | user | `Utilisateurs` |
| rôle | Rolle | ruolo | role | `Rôle` |
| paramètres | **Einstellungen** | **impostazioni** | settings | `Paramètres` |
| rapport | Bericht | rapporto | report | `Rapports comptables` |
| banque | Bank | banca | bank | `Banque` |
| devise | Währung | valuta | currency | `Devise non supportée v0.1.` |
| projet | Projekt | progetto | project | `depenses-par-projet` |
| dépenses | Ausgaben | spese | expenses | `depenses-par-projet` |

---

## B. Termes SANS précédent — arbitrage requis

⚠️ **Aucune de ces équivalences n'est attestée dans le catalogue.** Elles sont proposées,
avec leur motif. Une fois tranchées, elles remontent en partie A et deviennent
contraignantes pour tout le rollout.

| fr-CH | de-CH proposé | it-CH proposé | en-CH proposé | motif / réserve |
|---|---|---|---|---|
| QR-facture | **QR-Rechnung** | **fattura QR** | **QR-bill** | ✅ terminologie **officielle SIX**, pas un choix — à retenir tel quel |
| justificatif | **Beleg** | documento giustificativo | supporting document | `Beleg` est le terme du CO art. 957a en allemand ; « Buchungsbeleg » si le contexte est l'écriture |
| virement | Überweisung | bonifico | bank transfer | usage bancaire suisse standard |
| règlement (d'une facture) | Begleichung / Zahlung | pagamento | settlement | ⚠️ **préférer « Zahlung » / « pagamento » / « payment »** et réserver « règlement » au français — deux mots pour un concept déjà couvert par *paiement* |
| écarter / écartée (une pièce) | verwerfen / verworfen | scartare / scartata | discard / discarded | s'oppose à « compléter » dans la file d'import |
| compléter | vervollständigen | completare | complete | idem |
| analytique (compte, dimension) | **Kostenstelle** | **centro di costo** | cost center | ⚠️ **le vrai arbitrage de cette liste.** Kesh modélise l'analytique **par projet**, pas par centre de coûts. L'alternative est de ne jamais traduire « analytique » et de dire simplement *Projekt / progetto / project* — plus fidèle au modèle, moins conforme au vocabulaire comptable |
| localité | Ort | località | city | champ d'adresse |
| immuable | unveränderlich | immutabile | immutable | employé pour une pièce validée |
| validité | Gültigkeit | validità | validity | |
| personne physique / morale | natürliche / juristische Person | persona fisica / giuridica | individual / legal entity | |
| prénom | Vorname | nome | first name | |
| image | Bild | immagine | image | scan de QR-facture |
| bascule (interrupteur) | Umschalter | interruttore | toggle | élément d'interface |
| restauré | wiederhergestellt | ripristinato | restored | import de sauvegarde |

---

## Comment s'en servir

1. **Avant d'écrire une traduction**, chercher le terme ici. S'il y est, l'employer — sans
   variante « qui sonne mieux ».
2. **S'il n'y est pas et qu'il est récurrent**, l'ajouter en partie B plutôt que de trancher
   dans une story : le prochain rollout retrouvera la décision au lieu de la reprendre.
3. **Un terme de la partie A ne se change pas dans une story de rollout.** Il se change dans
   une story qui met AUSSI à jour toutes ses occurrences déjà livrées — sinon le produit
   dit deux mots pour une chose, ce que ce fichier existe pour empêcher.

[#316]: https://github.com/guycorbaz/kesh/issues/316
[#283]: https://github.com/guycorbaz/kesh/issues/283
