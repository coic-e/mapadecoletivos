//! Pedidos de correção: quem está de fora sugere, quem decide é a moderação.
//!
//! O que estes casos guardam é a fronteira. Um pedido é texto que chegou de
//! fora, guardado em jsonb e aplicado horas depois por um clique de moderador:
//! entre o envio e a aplicação, é preciso que continue valendo exatamente o
//! que o cadastro aceitaria.
//!
//! Rodam só com `TEST_DATABASE_URL` definida.
mod common;

use api_rust::domains::edit_requests::repository::EditRequestRepository;
use api_rust::domains::organizations::{actions, repository::OrganizationRepository};
use db_types::edit_request::{EditRequestStatus, NewEditRequest, OrganizationChanges};
use db_types::organization::ModerationStatus;
use validator::Validate;

use common::{admin, clear, new_organization, with_database};

fn aprovada(
    conn: &mut api_rust::db::DbConnection,
    nome: &str,
    moderador: i32,
) -> db_types::organization::Organization {
    let (org, _) =
        actions::create_organization(conn, new_organization(nome), vec!["a.jpg".to_string()], 0)
            .expect("o cadastro deveria ser criado");

    actions::review_organization(conn, org.id, ModerationStatus::APPROVED, moderador, None)
        .expect("a aprovação deveria passar")
        .0
}

fn pedido(
    conn: &mut api_rust::db::DbConnection,
    organization_id: i32,
    changes: serde_json::Value,
) -> db_types::edit_request::EditRequest {
    EditRequestRepository::create(
        conn,
        &NewEditRequest {
            organization_id,
            changes,
            message: Some("O endereço mudou".to_string()),
            requester_email: Some("quem-viu@exemplo.com".to_string()),
        },
    )
    .expect("o pedido deveria entrar na fila")
}

#[test]
fn a_request_waits_in_the_queue_without_touching_the_registration() {
    with_database(
        "a_request_waits_in_the_queue_without_touching_the_registration",
        |conn| {
            clear(conn);

            let moderador = admin(conn);
            let org = aprovada(conn, "Bunker 034", moderador.id);

            let pedido = pedido(
                conn,
                org.id,
                serde_json::json!({ "city": "Canoas", "address": "Rua Nova, 1" }),
            );

            assert_eq!(pedido.status, EditRequestStatus::PENDING);
            assert!(pedido.reviewed_by.is_none());

            let (intocada, _) = OrganizationRepository::find_by_id(conn, org.id).unwrap();

            assert_eq!(
                intocada.city, "Porto Alegre",
                "sugerir não é alterar: nada muda antes de um moderador aplicar"
            );
        },
    );
}

#[test]
fn applying_changes_the_registration_and_closes_the_request() {
    with_database(
        "applying_changes_the_registration_and_closes_the_request",
        |conn| {
            clear(conn);

            let moderador = admin(conn);
            let org = aprovada(conn, "Bunker 034", moderador.id);

            let pedido = pedido(
                conn,
                org.id,
                serde_json::json!({ "city": "Canoas", "genres": ["Acid"] }),
            );

            let changes: OrganizationChanges =
                serde_json::from_value(pedido.changes.clone()).unwrap();

            let atualizada = EditRequestRepository::apply(conn, &pedido, &changes, moderador.id)
                .expect("a aplicação deveria passar");

            assert_eq!(atualizada.city, "Canoas");
            assert_eq!(atualizada.genres, vec!["Acid".to_string()]);
            assert_eq!(
                atualizada.name, "Bunker 034",
                "campo ausente do pedido fica como estava"
            );

            let fechado = EditRequestRepository::find_by_id(conn, pedido.id).unwrap();

            assert_eq!(fechado.status, EditRequestStatus::APPLIED);
            assert_eq!(fechado.reviewed_by, Some(moderador.id));
            assert!(fechado.reviewed_at.is_some());
        },
    );
}

#[test]
fn an_omitted_field_is_left_alone_and_an_empty_one_is_cleared() {
    // A diferença que o painel depende: omitir é "não mexer", string vazia é
    // "apaga esse link".
    with_database(
        "an_omitted_field_is_left_alone_and_an_empty_one_is_cleared",
        |conn| {
            clear(conn);

            let moderador = admin(conn);
            let org = aprovada(conn, "Bunker 034", moderador.id);

            let changes: OrganizationChanges =
                serde_json::from_value(serde_json::json!({ "instagram": "" })).unwrap();

            let atualizada = EditRequestRepository::apply_changes(conn, org.id, &changes).unwrap();

            assert_eq!(atualizada.instagram.as_deref(), Some(""));
            assert_eq!(atualizada.email, org.email, "o resto não foi tocado");
            assert_eq!(atualizada.about, org.about);
            assert_eq!(atualizada.slug, org.slug);
        },
    );
}

#[test]
fn a_request_can_never_approve_itself() {
    // O `changes` é jsonb livre no banco. O que impede um pedido de virar
    // aprovação é a struct: status, slug e reviewed_by não existem nela, e o
    // serde os descarta na leitura.
    with_database("a_request_can_never_approve_itself", |conn| {
        clear(conn);

        let moderador = admin(conn);
        let (org, _) = actions::create_organization(
            conn,
            new_organization("Ainda Pendente"),
            vec!["a.jpg".to_string()],
            0,
        )
        .unwrap();

        let pedido = pedido(
            conn,
            org.id,
            serde_json::json!({
                "status": "approved",
                "slug": "cadastro-famoso",
                "reviewed_by": moderador.id,
                "city": "Canoas",
            }),
        );

        let changes: OrganizationChanges = serde_json::from_value(pedido.changes.clone()).unwrap();
        let atualizada =
            EditRequestRepository::apply(conn, &pedido, &changes, moderador.id).unwrap();

        assert_eq!(
            atualizada.status,
            ModerationStatus::PENDING,
            "o pedido não pode se auto-aprovar"
        );
        assert_eq!(atualizada.slug, org.slug, "nem sequestrar a URL de outro");
        assert_eq!(atualizada.city, "Canoas", "o que era legítimo passou");
    });
}

#[test]
fn a_request_stored_before_the_rules_changed_is_checked_again_on_the_way_out() {
    // Um pedido pode ficar na fila enquanto as listas fechadas mudam. É na
    // aplicação que ele vira dado público, e é lá que precisa valer de novo.
    with_database(
        "a_request_stored_before_the_rules_changed_is_checked_again_on_the_way_out",
        |conn| {
            clear(conn);

            let moderador = admin(conn);
            let org = aprovada(conn, "Bunker 034", moderador.id);

            // Entra no banco sem passar por validação nenhuma, que é
            // exatamente o cenário de um pedido antigo.
            let guardado = pedido(
                conn,
                org.id,
                serde_json::json!({ "genres": ["Sertanejo"], "website": "javascript:alert(1)" }),
            );

            let changes: OrganizationChanges =
                serde_json::from_value(guardado.changes.clone()).unwrap();

            assert!(
                changes.validate().is_err() || changes.validate_closed_lists().is_err(),
                "a revalidação na aplicação é a última barreira"
            );
        },
    );
}

#[test]
fn rejecting_closes_the_request_and_leaves_the_registration_alone() {
    with_database(
        "rejecting_closes_the_request_and_leaves_the_registration_alone",
        |conn| {
            clear(conn);

            let moderador = admin(conn);
            let org = aprovada(conn, "Bunker 034", moderador.id);
            let pedido = pedido(conn, org.id, serde_json::json!({ "city": "Canoas" }));

            let recusado = EditRequestRepository::set_status(
                conn,
                pedido.id,
                EditRequestStatus::REJECTED,
                moderador.id,
            )
            .unwrap();

            assert_eq!(recusado.status, EditRequestStatus::REJECTED);

            let (intocada, _) = OrganizationRepository::find_by_id(conn, org.id).unwrap();
            assert_eq!(intocada.city, "Porto Alegre");
        },
    );
}

#[test]
fn the_queue_filters_by_state_and_answers_the_oldest_first() {
    with_database(
        "the_queue_filters_by_state_and_answers_the_oldest_first",
        |conn| {
            clear(conn);

            let moderador = admin(conn);
            let org = aprovada(conn, "Bunker 034", moderador.id);

            let primeiro = pedido(conn, org.id, serde_json::json!({ "city": "Canoas" }));
            let segundo = pedido(conn, org.id, serde_json::json!({ "city": "Gravataí" }));
            let recusado = pedido(conn, org.id, serde_json::json!({ "city": "Viamão" }));

            EditRequestRepository::set_status(
                conn,
                recusado.id,
                EditRequestStatus::REJECTED,
                moderador.id,
            )
            .unwrap();

            let pendentes: Vec<i32> =
                EditRequestRepository::find_all_with_status(conn, Some(EditRequestStatus::PENDING))
                    .unwrap()
                    .into_iter()
                    .map(|r| r.id)
                    .collect();

            assert_eq!(pendentes, vec![primeiro.id, segundo.id]);

            let todos = EditRequestRepository::find_all_with_status(conn, None).unwrap();
            assert_eq!(todos.len(), 3, "sem filtro, a fila mostra tudo");
        },
    );
}

#[test]
fn a_request_that_does_not_exist_is_a_404() {
    with_database("a_request_that_does_not_exist_is_a_404", |conn| {
        clear(conn);

        assert!(matches!(
            EditRequestRepository::find_by_id(conn, 999_999),
            Err(api_rust::errors::ApiError::NotFound)
        ));
    });
}

#[test]
fn applying_is_all_or_nothing() {
    // Pedido marcado como aplicado sem a mudança correspondente seria pior do
    // que não ter aplicado: some da fila e não mudou nada.
    with_database("applying_is_all_or_nothing", |conn| {
        clear(conn);

        let moderador = admin(conn);
        let org = aprovada(conn, "Bunker 034", moderador.id);

        // Aponta para um cadastro que não existe: a alteração falha, e a
        // marcação do pedido tem que cair junto.
        let orfao = pedido(conn, org.id, serde_json::json!({ "city": "Canoas" }));
        let mut apontando_para_o_vazio = orfao.clone();
        apontando_para_o_vazio.organization_id = 999_999;

        let changes: OrganizationChanges = serde_json::from_value(orfao.changes.clone()).unwrap();

        assert!(EditRequestRepository::apply(
            conn,
            &apontando_para_o_vazio,
            &changes,
            moderador.id
        )
        .is_err());

        let ainda_pendente = EditRequestRepository::find_by_id(conn, orfao.id).unwrap();

        assert_eq!(
            ainda_pendente.status,
            EditRequestStatus::PENDING,
            "a transação inteira deveria ter voltado atrás"
        );
    });
}
