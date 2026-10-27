//! `kesh_contacts` et `kesh_contact` — le carnet d'adresses (Story 32-9b,
//! refs #723, AC-b2). Portée `contacts:read`.
//!
//! La portée **est** celle des données personnelles : chaque texte du contact
//! — nom, prénom, adresse, courriel, téléphone, numéro IDE, numéro de client,
//! conditions — sort **enveloppé**, sans masquage (le masquage des tiers vise
//! les clés qui n'ont **pas** `contacts:read`, C-32-9-9). Ne sont pas rendus :
//! la société de l'installation (`PageContacts.societe`), la version et les
//! horodatages.

use serde::Deserialize;

use crate::mcp::outil::{ContexteMcp, FuturOutil, McpTool, arguments};
use crate::mcp::outils::commun::{SensDuTri, decalage, limite, recherche_bornee};
use crate::mcp::sortie::{Resume, TexteNonFiable, donnees, sortie};
use crate::mcp::textes::depuis_app_error;
use crate::services::acces::Portee;
use crate::services::carnet::{self, ContactSortBy, DemandeListeContacts, FicheContact};
use kesh_db::entities::Language;
use kesh_db::entities::contact::{Contact, ContactType, Salutation};

/// Taille de page par défaut (§ *Les tailles mesurées* de la fiche).
pub const LIMITE_DEFAUT: i64 = 12;
/// Taille de page maximale de l'outil (le service admet 100).
pub const LIMITE_MAX_MCP: i64 = 22;

/// La liste.
pub struct KeshContacts;
/// La fiche.
pub struct KeshContact;

/// Le type de contact, en argument.
#[derive(Debug, Clone, Copy, Deserialize)]
enum TypeArg {
    Personne,
    Entreprise,
}

impl TypeArg {
    fn du_service(self) -> ContactType {
        match self {
            Self::Personne => ContactType::Personne,
            Self::Entreprise => ContactType::Entreprise,
        }
    }

    fn code(self) -> &'static str {
        match self {
            Self::Personne => "Personne",
            Self::Entreprise => "Entreprise",
        }
    }
}

/// La colonne de tri, en argument.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
enum TriContacts {
    Name,
    CreatedAt,
    UpdatedAt,
}

impl TriContacts {
    fn du_service(self) -> ContactSortBy {
        match self {
            Self::Name => ContactSortBy::Name,
            Self::CreatedAt => ContactSortBy::CreatedAt,
            Self::UpdatedAt => ContactSortBy::UpdatedAt,
        }
    }

    fn code(self) -> &'static str {
        match self {
            Self::Name => "name",
            Self::CreatedAt => "createdAt",
            Self::UpdatedAt => "updatedAt",
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct ArgumentsListe {
    search: Option<String>,
    contact_type: Option<TypeArg>,
    is_client: Option<bool>,
    is_supplier: Option<bool>,
    include_archived: Option<bool>,
    sort_by: Option<TriContacts>,
    sort_direction: Option<SensDuTri>,
    limit: Option<i64>,
    offset: Option<i64>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ArgumentsDetail {
    id: i64,
}

sortie! {
    /// Un contact — la même forme dans la liste et la fiche.
    struct ContactSortie {
        id: i64,
        type_de_contact: ContactType,
        nom: TexteNonFiable,
        prenom: Option<TexteNonFiable>,
        nom_de_famille: Option<TexteNonFiable>,
        client: bool,
        fournisseur: bool,
        adresse: Option<TexteNonFiable>,
        rue: Option<TexteNonFiable>,
        batiment: Option<TexteNonFiable>,
        npa: Option<TexteNonFiable>,
        localite: Option<TexteNonFiable>,
        pays: Option<TexteNonFiable>,
        email: Option<TexteNonFiable>,
        telephone: Option<TexteNonFiable>,
        ide: Option<TexteNonFiable>,
        numero_client: Option<TexteNonFiable>,
        conditions: Option<TexteNonFiable>,
        delai_paiement_jours: Option<i32>,
        langue: Option<Language>,
        civilite: Salutation,
        actif: bool,
    }
}

sortie! {
    /// La liste.
    struct Contacts { contacts: Vec<ContactSortie>, total: i64, limit: i64, offset: i64 }
}

sortie! {
    /// La fiche.
    struct FicheSortie { contact: ContactSortie }
}

/// La projection d'un contact. ⚠️ Déstructuration **exhaustive**, sans `..` :
/// un champ ajouté à l'entité ne compile pas ici tant qu'on n'a pas décidé
/// s'il atteint l'agent.
fn contact_sortie(c: Contact) -> ContactSortie {
    let Contact {
        id,
        company_id: _,
        contact_type,
        name,
        first_name,
        last_name,
        is_client,
        is_supplier,
        address,
        address_street,
        address_building,
        address_postal_code,
        address_city,
        address_country,
        email,
        phone,
        ide_number,
        client_number,
        default_payment_terms,
        default_payment_terms_days,
        language,
        salutation,
        active,
        version: _,
        created_at: _,
        updated_at: _,
    } = c;
    ContactSortie {
        id,
        type_de_contact: contact_type,
        nom: TexteNonFiable::de(name),
        prenom: first_name.map(TexteNonFiable::de),
        nom_de_famille: last_name.map(TexteNonFiable::de),
        client: is_client,
        fournisseur: is_supplier,
        adresse: address.map(TexteNonFiable::de),
        rue: address_street.map(TexteNonFiable::de),
        batiment: address_building.map(TexteNonFiable::de),
        npa: address_postal_code.map(TexteNonFiable::de),
        localite: address_city.map(TexteNonFiable::de),
        pays: address_country.map(TexteNonFiable::de),
        email: email.map(TexteNonFiable::de),
        telephone: phone.map(TexteNonFiable::de),
        ide: ide_number.map(TexteNonFiable::de),
        numero_client: client_number.map(TexteNonFiable::de),
        conditions: default_payment_terms.map(TexteNonFiable::de),
        delai_paiement_jours: default_payment_terms_days,
        langue: language,
        civilite: salutation,
        actif: active,
    }
}

impl McpTool for KeshContacts {
    fn nom(&self) -> &'static str {
        "kesh_contacts"
    }

    fn description(&self) -> &'static str {
        "Liste les contacts du carnet d'adresses : nom, adresse, courriel, téléphone, numéro \
         IDE, numéro de client, conditions de paiement ; filtrés (recherche, type, clients, \
         fournisseurs, archivés), triés et paginés (limit, offset). Si la réponse est trop \
         grosse, réduisez limit."
    }

    fn schema_entree(&self) -> &'static str {
        r#"{"type":"object","properties":{"search":{"type":"string","maxLength":100},"contactType":{"type":"string","enum":["Personne","Entreprise"]},"isClient":{"type":"boolean"},"isSupplier":{"type":"boolean"},"includeArchived":{"type":"boolean"},"sortBy":{"type":"string","enum":["name","createdAt","updatedAt"]},"sortDirection":{"type":"string","enum":["asc","desc"]},"limit":{"type":"integer","minimum":1,"maximum":22,"default":12},"offset":{"type":"integer","minimum":0}},"additionalProperties":false}"#
    }

    fn portee(&self) -> Portee {
        Portee::ContactsRead
    }

    fn executer<'a>(
        &'a self,
        ctx: &'a ContexteMcp<'a>,
        valeur: serde_json::Value,
    ) -> FuturOutil<'a> {
        Box::pin(async move {
            let a: ArgumentsListe = arguments(valeur)?;
            let limit = limite(a.limit, LIMITE_DEFAUT, LIMITE_MAX_MCP)?;
            let offset = decalage(a.offset)?;
            // Le service du carnet ne borne pas la recherche (l'écran est
            // inchangé) ; l'outil la borne comme `kesh_factures` (C-32-9-64).
            let search = recherche_bornee(a.search)?;
            let resume = Resume::outil("kesh_contacts")
                .recherche_fournie("search", search.is_some())
                .code("contactType", a.contact_type.map(TypeArg::code))
                .booleen("isClient", a.is_client)
                .booleen("isSupplier", a.is_supplier)
                .booleen("includeArchived", a.include_archived)
                .code("sortBy", a.sort_by.map(TriContacts::code))
                .code("sortDirection", a.sort_direction.map(SensDuTri::code))
                .entier("limit", Some(limit))
                .entier("offset", Some(offset));
            let demande = DemandeListeContacts {
                search,
                contact_type: a.contact_type.map(TypeArg::du_service),
                is_client: a.is_client,
                is_supplier: a.is_supplier,
                include_archived: a.include_archived.unwrap_or(false),
                sort_by: a.sort_by.map(TriContacts::du_service),
                sort_direction: a.sort_direction.map(SensDuTri::du_service),
                limit: Some(limit),
                offset: Some(offset),
            };
            let page = carnet::lister(ctx.acces(), ctx.lecture(), demande)
                .await
                .map_err(depuis_app_error)?
                .page;
            let sortie = Contacts {
                contacts: page.items.into_iter().map(contact_sortie).collect(),
                total: page.total,
                limit: page.limit,
                offset: page.offset,
            };
            donnees(resume, &sortie)
        })
    }
}

impl McpTool for KeshContact {
    fn nom(&self) -> &'static str {
        "kesh_contact"
    }

    fn description(&self) -> &'static str {
        "Lit un contact du carnet d'adresses par son identifiant : nom, adresse, courriel, \
         téléphone, numéro IDE, numéro de client, conditions et délai de paiement, langue, \
         civilité."
    }

    fn schema_entree(&self) -> &'static str {
        r#"{"type":"object","properties":{"id":{"type":"integer","minimum":1}},"required":["id"],"additionalProperties":false}"#
    }

    fn portee(&self) -> Portee {
        Portee::ContactsRead
    }

    fn executer<'a>(
        &'a self,
        ctx: &'a ContexteMcp<'a>,
        valeur: serde_json::Value,
    ) -> FuturOutil<'a> {
        Box::pin(async move {
            let ArgumentsDetail { id } = arguments(valeur)?;
            let FicheContact { contact, .. } = carnet::detail(ctx.acces(), ctx.lecture(), id)
                .await
                .map_err(depuis_app_error)?;
            donnees(
                Resume::outil("kesh_contact").entier("id", Some(id)),
                &FicheSortie {
                    contact: contact_sortie(contact),
                },
            )
        })
    }
}
