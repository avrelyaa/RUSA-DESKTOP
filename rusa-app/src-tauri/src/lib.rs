pub mod db;
pub mod models;
pub mod queries;
pub mod auth;
pub mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let _app_handle = app.handle().clone();
            tauri::async_runtime::block_on(async move {
                let database_url = std::env::var("DATABASE_URL")
                    .unwrap_or_else(|_| "postgres://rusa:rusa@localhost:5432/rusa_ims".to_string());
                let redis_url = std::env::var("REDIS_URL")
                    .unwrap_or_else(|_| "redis://127.0.0.1/".to_string());

                db::init_db(&database_url).await.expect("Failed to init DB");
                db::init_redis(&redis_url).expect("Failed to init Redis");
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // auth
            commands::auth_commands::login,
            commands::auth_commands::logout,
            commands::auth_commands::get_current_user,
            // messages
            commands::message_commands::send_message,
            commands::message_commands::get_inbox,
            commands::message_commands::get_sent,
            commands::message_commands::get_message,
            commands::message_commands::recall_message,
            commands::message_commands::mark_message_read,
            // users
            commands::user_commands::get_all_users,
            commands::user_commands::get_roles,
            commands::user_commands::create_user,
            commands::user_commands::deactivate_user,
            commands::user_commands::delete_user,
            // research
            commands::research_commands::get_experiments,
            commands::research_commands::propose_experiment,
            commands::research_commands::review_experiment,
            commands::research_commands::add_experiment_log,
            commands::research_commands::get_species_archive,
            commands::research_commands::add_species,
            commands::research_commands::get_test_archive,
            commands::research_commands::propose_test,
            commands::research_commands::review_test_proposal,
            // data
            commands::data_commands::submit_data_request,
            commands::data_commands::get_data_requests,
            commands::data_commands::review_data_request,
            commands::data_commands::respond_to_data_request,
            commands::data_commands::deliver_data_response,
            // security
            commands::security_commands::create_incident_report,
            commands::security_commands::get_incident_reports,
            commands::security_commands::update_incident_status,
            commands::security_commands::add_lost_found_item,
            commands::security_commands::get_lost_found,
            commands::security_commands::claim_lost_found,
            commands::security_commands::submit_broadcast_request,
            commands::security_commands::get_broadcast_requests,
            commands::security_commands::review_broadcast_request,
            commands::security_commands::submit_security_findings,
            commands::security_commands::get_security_findings,
            // astronaut
            commands::astronaut_commands::get_missions,
            commands::astronaut_commands::create_mission,
            commands::astronaut_commands::update_mission_status,
            commands::astronaut_commands::assign_crew,
            commands::astronaut_commands::submit_mission_report,
            commands::astronaut_commands::get_mission_reports,
            commands::astronaut_commands::create_exploration_journal,
            commands::astronaut_commands::get_journals,
            commands::astronaut_commands::get_sectors,
            commands::astronaut_commands::create_sector,
            commands::astronaut_commands::rename_sector,
            commands::astronaut_commands::get_ships,
            commands::astronaut_commands::get_astronaut_stats,
            // settlement
            commands::settlement_commands::get_settlements,
            commands::settlement_commands::get_settler_tasks,
            commands::settlement_commands::assign_settler_task,
            commands::settlement_commands::update_task_progress,
            commands::settlement_commands::submit_supply_request,
            commands::settlement_commands::review_supply_request,
            commands::settlement_commands::submit_anomaly_report,
            commands::settlement_commands::review_anomaly_report,
            commands::settlement_commands::issue_house_arrest,
            commands::settlement_commands::submit_send_to_earth,
            commands::settlement_commands::review_send_to_earth,
            commands::settlement_commands::log_settlement_inventory,
            commands::settlement_commands::get_settlement_inventory,
            commands::settlement_commands::submit_farm_report,
            commands::settlement_commands::get_supply_requests,
            commands::settlement_commands::get_anomaly_reports,
            // station
            commands::station_commands::get_stations,
            commands::station_commands::get_station_inventory,
            commands::station_commands::update_station_inventory,
            commands::station_commands::add_map_annotation,
            commands::station_commands::get_map_annotations,
            commands::station_commands::log_personnel_event,
            commands::station_commands::get_personnel_log,
            commands::station_commands::submit_station_findings,
            commands::station_commands::get_station_findings,
            commands::station_commands::submit_station_supply_request,
            commands::station_commands::review_station_supply_request,
            commands::station_commands::submit_station_abandonment,
            // psychiatry
            commands::psychiatry_commands::register_patient,
            commands::psychiatry_commands::get_patients,
            commands::psychiatry_commands::schedule_appointment,
            commands::psychiatry_commands::get_appointments,
            commands::psychiatry_commands::complete_appointment,
            commands::psychiatry_commands::add_recovery_log,
            commands::psychiatry_commands::get_recovery_log,
            // medical
            commands::medical_commands::allocate_shift,
            commands::medical_commands::get_shifts,
            commands::medical_commands::get_medical_inventory,
            commands::medical_commands::update_medical_inventory,
            commands::medical_commands::create_patient_record,
            commands::medical_commands::get_patient_records,
            commands::medical_commands::add_specialization,
            commands::medical_commands::get_staff_specializations,
            commands::medical_commands::submit_budget_request,
            commands::medical_commands::submit_expenditure_report,
            // sanitary
            commands::sanitary_commands::get_sanitary_tasks,
            commands::sanitary_commands::assign_sanitary_task,
            commands::sanitary_commands::update_sanitary_task,
            commands::sanitary_commands::get_sanitary_inventory,
            commands::sanitary_commands::update_sanitary_inventory,
            commands::sanitary_commands::add_disposal_log,
            commands::sanitary_commands::get_disposal_logs,
            commands::sanitary_commands::add_wastewater_log,
            commands::sanitary_commands::get_wastewater_logs,
            commands::sanitary_commands::submit_division_transfer,
            commands::sanitary_commands::review_division_transfer,
            commands::sanitary_commands::set_division_quota,
            commands::sanitary_commands::create_inspection_report,
            commands::sanitary_commands::get_inspection_reports,
            commands::sanitary_commands::send_inspection_to_head,
            // governance
            commands::governance_commands::initiate_vote,
            commands::governance_commands::cast_vote,
            commands::governance_commands::get_votes,
            commands::governance_commands::get_vote_details,
            commands::governance_commands::interrupt_vote,
            commands::governance_commands::create_meeting,
            commands::governance_commands::get_meetings,
            commands::governance_commands::relocate_staff,
            commands::governance_commands::get_relocations,
            commands::governance_commands::set_archive_permission,
            commands::governance_commands::log_event_document,
            commands::governance_commands::get_event_documents,
            // budget
            commands::budget_commands::get_budget_requests,
            commands::budget_commands::review_budget_request,
            commands::budget_commands::flag_budget_request,
            commands::budget_commands::get_expenditure_reports,
            commands::budget_commands::flag_expenditure_report,
            commands::budget_commands::submit_investigation,
            // admin
            commands::admin_commands::admin_create_director,
            commands::admin_commands::admin_terminate_director,
            commands::admin_commands::terminate_personnel,
            commands::admin_commands::get_audit_log,
            commands::admin_commands::override_vote,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

