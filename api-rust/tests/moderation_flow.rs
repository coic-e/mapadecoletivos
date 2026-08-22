//! O caminho de um cadastro, do envio ao mapa, contra um Postgres de verdade.
//!
//! Estes casos existem porque a regra que mais importa aqui — cadastro só
//! existe para o público depois de aprovado — mora numa cláusula de SQL, e
//! nenhum teste sem banco consegue afirmar que ela está lá.
//!
//! Rodam só com `TEST_DATABASE_URL` definida, num banco descartável. Sem ela,
//! cada caso se anuncia como pulado e passa.
mod common;

use api_rust::domains::organizations::actions;
use api_rust::domains::organizations::repository::OrganizationRepository;
use api_rust::errors::ApiError;
use db_types::organization::ModerationStatus;

use common::{admin, clear, new_organization, with_database};

/// Cria um cadastro com as imagens dadas e devolve o que o banco gravou.
fn criar(
    conn: &mut api_rust::db::DbConnection,
    nome: &str,
    imagens: &[&str],
    capa: usize,
) -> (
    db_types::organization::Organization,
    Vec<db_types::image::Image>,
) {
    actions::create_organization(
        conn,
        new_organization(nome),
        imagens.iter().map(|s| s.to_string()).collect(),
        capa,
    )
    .expect("o cadastro deveria ser criado")
}

fn aprovar(conn: &mut api_rust::db::DbConnection, id: i32, moderador: i32) {
    actions::review_organization(conn, id, ModerationStatus::APPROVED, moderador, None)
        .expect("a aprovação deveria passar");
}

#[test]
fn a_new_registration_is_born_pending_and_invisible() {
    // O cadastro é aberto: se aparecesse no mapa antes da revisão, qualquer
    // um publicaria o que quisesse no site.
    with_database("a_new_registration_is_born_pending_and_invisible", |conn| {
        clear(conn);

        let (org, _) = criar(conn, "Bunker 034", &["a.jpg"], 0);

        assert_eq!(org.status, ModerationStatus::PENDING);
        assert!(org.reviewed_at.is_none());
        assert!(org.reviewed_by.is_none());

        assert!(
            actions::get_all_organizations(conn, Some(50), Some(0))
                .unwrap()
                .is_empty(),
            "pendente não entra na listagem pública"
        );

        assert!(
            matches!(
                actions::get_organization_by_id(conn, org.id),
                Err(ApiError::NotFound)
            ),
            "pendente responde 404, e não \"existe mas está escondido\""
        );

        assert!(matches!(
            actions::get_organization_by_slug(conn, &org.slug),
            Err(ApiError::NotFound)
        ));
    });
}

#[test]
fn approving_puts_it_on_the_map() {
    with_database("approving_puts_it_on_the_map", |conn| {
        clear(conn);

        let moderador = admin(conn);
        let (org, _) = criar(conn, "Bunker 034", &["a.jpg"], 0);

        aprovar(conn, org.id, moderador.id);

        let (publico, imagens) =
            actions::get_organization_by_id(conn, org.id).expect("aprovado deveria aparecer");

        assert_eq!(publico.status, ModerationStatus::APPROVED);
        assert_eq!(publico.reviewed_by, Some(moderador.id));
        assert!(publico.reviewed_at.is_some(), "a decisão fica datada");
        assert_eq!(imagens.len(), 1);

        let (por_slug, _) = actions::get_organization_by_slug(conn, &publico.slug)
            .expect("o slug também deveria achar");

        assert_eq!(por_slug.id, org.id);

        assert_eq!(
            actions::get_all_organizations(conn, Some(50), Some(0))
                .unwrap()
                .len(),
            1
        );
    });
}

#[test]
fn rejecting_takes_it_off_the_map_and_throws_the_photos_away() {
    // Cadastro rejeitado não volta ao site, e guardar as fotos só acumula
    // custo: um envio automatizado empurra dezenas de megabytes por vez.
    with_database(
        "rejecting_takes_it_off_the_map_and_throws_the_photos_away",
        |conn| {
            clear(conn);

            let moderador = admin(conn);
            let (org, _) = criar(conn, "Spam Automatizado", &["a.jpg", "b.png"], 0);

            let (rejeitada, imagens, descartadas) = actions::review_organization(
                conn,
                org.id,
                ModerationStatus::REJECTED,
                moderador.id,
                Some("Fora do escopo do mapa".to_string()),
            )
            .expect("a rejeição deveria passar");

            assert_eq!(rejeitada.status, ModerationStatus::REJECTED);
            assert_eq!(
                rejeitada.rejection_reason.as_deref(),
                Some("Fora do escopo do mapa")
            );

            let mut chaves = descartadas.clone();
            chaves.sort();

            assert_eq!(
                chaves,
                vec!["a.jpg".to_string(), "b.png".to_string()],
                "as chaves precisam voltar para a rota apagar do bucket"
            );
            assert!(imagens.is_empty(), "as linhas de imagem saíram do banco");

            assert!(matches!(
                actions::get_organization_by_id(conn, org.id),
                Err(ApiError::NotFound)
            ));
        },
    );
}

#[test]
fn approving_keeps_the_photos() {
    with_database("approving_keeps_the_photos", |conn| {
        clear(conn);

        let moderador = admin(conn);
        let (org, _) = criar(conn, "Bunker 034", &["a.jpg", "b.png"], 0);

        let (_, imagens, descartadas) = actions::review_organization(
            conn,
            org.id,
            ModerationStatus::APPROVED,
            moderador.id,
            None,
        )
        .unwrap();

        assert_eq!(imagens.len(), 2);
        assert!(
            descartadas.is_empty(),
            "nada a apagar do bucket numa aprovação"
        );
    });
}

#[test]
fn a_registration_that_does_not_exist_is_a_404_not_a_database_error() {
    with_database(
        "a_registration_that_does_not_exist_is_a_404_not_a_database_error",
        |conn| {
            clear(conn);

            let moderador = admin(conn);

            assert!(matches!(
                actions::review_organization(
                    conn,
                    999_999,
                    ModerationStatus::APPROVED,
                    moderador.id,
                    None
                ),
                Err(ApiError::NotFound)
            ));
        },
    );
}

#[test]
fn an_invented_status_never_reaches_the_database() {
    // A coluna tem CHECK, mas o erro do banco voltaria como 500. Aqui vira
    // erro de validação antes de qualquer escrita.
    with_database("an_invented_status_never_reaches_the_database", |conn| {
        clear(conn);

        let moderador = admin(conn);
        let (org, _) = criar(conn, "Bunker 034", &["a.jpg"], 0);

        for inventado in ["aprovado", "APPROVED", "applied", "", "all"] {
            let erro = actions::review_organization(conn, org.id, inventado, moderador.id, None)
                .expect_err(&format!("{inventado:?} não é estado de moderação"));

            assert!(
                matches!(erro, ApiError::ValidationError(_)),
                "{inventado:?} deu {erro:?}"
            );
        }

        let (intocada, _) = OrganizationRepository::find_by_id(conn, org.id).unwrap();
        assert_eq!(intocada.status, ModerationStatus::PENDING);
    });
}

#[test]
fn two_registrations_with_the_same_name_get_different_urls() {
    // O slug tem índice único e vai para a URL: sem o sufixo, o segundo
    // cadastro com o mesmo nome falharia na inserção.
    with_database(
        "two_registrations_with_the_same_name_get_different_urls",
        |conn| {
            clear(conn);

            let (primeira, _) = criar(conn, "Bunker 034", &["a.jpg"], 0);
            let (segunda, _) = criar(conn, "Bunker 034", &["b.jpg"], 0);
            let (terceira, _) = criar(conn, "bunker 034!", &["c.jpg"], 0);

            assert_eq!(primeira.slug, "bunker-034");
            assert_eq!(segunda.slug, "bunker-034-2");
            assert_eq!(terceira.slug, "bunker-034-3");
        },
    );
}

#[test]
fn the_chosen_photo_becomes_the_cover() {
    with_database("the_chosen_photo_becomes_the_cover", |conn| {
        clear(conn);

        let moderador = admin(conn);
        // A terceira foto enviada é a capa.
        let (org, _) = criar(conn, "Bunker 034", &["a.jpg", "b.jpg", "c.jpg"], 2);

        aprovar(conn, org.id, moderador.id);

        let (_, imagens) = actions::get_organization_by_id(conn, org.id).unwrap();

        assert_eq!(
            imagens.iter().map(|i| i.path.as_str()).collect::<Vec<_>>(),
            vec!["c.jpg", "a.jpg", "b.jpg"],
            "a capa vem primeiro; as outras seguem a ordem de envio"
        );
    });
}

#[test]
fn a_cover_index_out_of_range_falls_back_to_the_first_photo() {
    with_database(
        "a_cover_index_out_of_range_falls_back_to_the_first_photo",
        |conn| {
            clear(conn);

            let moderador = admin(conn);
            let (org, _) = criar(conn, "Bunker 034", &["a.jpg", "b.jpg"], 99);

            aprovar(conn, org.id, moderador.id);

            let (_, imagens) = actions::get_organization_by_id(conn, org.id).unwrap();

            assert_eq!(imagens[0].path, "a.jpg");
            assert_eq!(imagens[0].position, 0);
        },
    );
}

#[test]
fn the_moderation_queue_filters_by_state() {
    with_database("the_moderation_queue_filters_by_state", |conn| {
        clear(conn);

        let moderador = admin(conn);

        let (pendente, _) = criar(conn, "Pendente", &["a.jpg"], 0);
        let (aprovada, _) = criar(conn, "Aprovada", &["b.jpg"], 0);
        let (rejeitada, _) = criar(conn, "Rejeitada", &["c.jpg"], 0);

        aprovar(conn, aprovada.id, moderador.id);
        actions::review_organization(
            conn,
            rejeitada.id,
            ModerationStatus::REJECTED,
            moderador.id,
            None,
        )
        .unwrap();

        let mut ids = |status: Option<&str>| {
            actions::get_organizations_for_moderation(
                conn,
                status.map(str::to_string),
                Some(50),
                Some(0),
            )
            .unwrap()
            .into_iter()
            .map(|(org, _)| org.id)
            .collect::<Vec<_>>()
        };

        assert_eq!(ids(Some(ModerationStatus::PENDING)), vec![pendente.id]);
        assert_eq!(ids(Some(ModerationStatus::APPROVED)), vec![aprovada.id]);
        assert_eq!(ids(Some(ModerationStatus::REJECTED)), vec![rejeitada.id]);
        assert_eq!(
            ids(None).len(),
            3,
            "sem filtro, a fila mostra todos os estados"
        );
    });
}

#[test]
fn the_moderation_queue_shows_every_photo_of_a_registration() {
    // Regressão: a fila mostrava só uma foto por cadastro, e a moderação
    // decidia sem ver o resto do que seria publicado.
    with_database(
        "the_moderation_queue_shows_every_photo_of_a_registration",
        |conn| {
            clear(conn);

            criar(conn, "Bunker 034", &["a.jpg", "b.jpg", "c.jpg"], 0);

            let fila = actions::get_organizations_for_moderation(
                conn,
                Some(ModerationStatus::PENDING.to_string()),
                Some(50),
                Some(0),
            )
            .unwrap();

            assert_eq!(fila.len(), 1);
            assert_eq!(
                fila[0].1.len(),
                3,
                "as três fotos precisam chegar ao painel"
            );
        },
    );
}

#[test]
fn the_queue_answers_the_oldest_first() {
    // Quem cadastrou antes espera menos.
    with_database("the_queue_answers_the_oldest_first", |conn| {
        clear(conn);

        let (primeira, _) = criar(conn, "Primeira", &["a.jpg"], 0);
        let (segunda, _) = criar(conn, "Segunda", &["b.jpg"], 0);

        let fila: Vec<i32> =
            actions::get_organizations_for_moderation(conn, None, Some(50), Some(0))
                .unwrap()
                .into_iter()
                .map(|(org, _)| org.id)
                .collect();

        assert_eq!(fila, vec![primeira.id, segunda.id]);
    });
}

#[test]
fn pagination_walks_the_list_without_repeating_or_skipping() {
    with_database(
        "pagination_walks_the_list_without_repeating_or_skipping",
        |conn| {
            clear(conn);

            let moderador = admin(conn);

            let mut ids = Vec::new();
            for i in 0..5 {
                let (org, _) = criar(conn, &format!("Coletivo {i}"), &["a.jpg"], 0);
                aprovar(conn, org.id, moderador.id);
                ids.push(org.id);
            }

            let mut pagina = |limit, offset| {
                actions::get_all_organizations(conn, Some(limit), Some(offset))
                    .unwrap()
                    .into_iter()
                    .map(|(org, _)| org.id)
                    .collect::<Vec<_>>()
            };

            assert_eq!(pagina(2, 0), ids[0..2]);
            assert_eq!(pagina(2, 2), ids[2..4]);
            assert_eq!(pagina(2, 4), ids[4..5]);
            assert!(pagina(2, 10).is_empty(), "depois do fim, página vazia");
        },
    );
}

#[test]
fn every_photo_of_a_registration_comes_back_in_order() {
    with_database(
        "every_photo_of_a_registration_comes_back_in_order",
        |conn| {
            clear(conn);

            let moderador = admin(conn);
            let (org, _) = criar(conn, "Bunker 034", &["a.jpg", "b.jpg", "c.jpg", "d.jpg"], 1);

            aprovar(conn, org.id, moderador.id);

            let (_, imagens) = actions::get_organization_by_id(conn, org.id).unwrap();
            let posicoes: Vec<i32> = imagens.iter().map(|i| i.position).collect();

            assert_eq!(imagens.len(), 4);
            assert_eq!(posicoes, vec![0, 1, 2, 3], "sem buraco nem repetição");
        },
    );
}

#[test]
fn one_registration_never_carries_another_ones_photos() {
    with_database(
        "one_registration_never_carries_another_ones_photos",
        |conn| {
            clear(conn);

            let moderador = admin(conn);
            let (primeira, _) = criar(conn, "Primeira", &["a.jpg", "b.jpg"], 0);
            let (segunda, _) = criar(conn, "Segunda", &["c.jpg"], 0);

            aprovar(conn, primeira.id, moderador.id);
            aprovar(conn, segunda.id, moderador.id);

            let (_, da_primeira) = actions::get_organization_by_id(conn, primeira.id).unwrap();
            let (_, da_segunda) = actions::get_organization_by_id(conn, segunda.id).unwrap();

            assert_eq!(
                da_primeira
                    .iter()
                    .map(|i| i.path.as_str())
                    .collect::<Vec<_>>(),
                vec!["a.jpg", "b.jpg"]
            );
            assert_eq!(
                da_segunda
                    .iter()
                    .map(|i| i.path.as_str())
                    .collect::<Vec<_>>(),
                vec!["c.jpg"]
            );
        },
    );
}
