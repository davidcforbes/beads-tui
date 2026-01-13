use beads_tui::models::AppState;
use beads_tui::ui::views::DatabaseViewMode;
use proptest::prelude::*;

// Optimize by reducing iterations and reusing state if possible,
// though proptest encourages fresh state.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))] // Fewer cases for expensive state init
    #[test]
    fn test_app_state_transitions(
        steps in proptest::collection::vec(0..10u8, 1..100)
    ) {
        let mut app = AppState::new();

        for step in steps {
            match step {
                0 => app.next_tab(),
                1 => app.previous_tab(),
                2 => app.toggle_perf_stats(),
                3 => app.next_help_section(),
                4 => app.previous_help_section(),
                5 => {
                    let modes = DatabaseViewMode::all();
                    let current_idx = modes.iter().position(|m| *m == app.database_view_state.mode).unwrap_or(0);
                    let next_idx = (current_idx + 1) % modes.len();
                    app.database_view_state.set_mode(modes[next_idx]);
                },
                6 => {
                    if app.selected_molecular_tab > 0 {
                        app.selected_molecular_tab -= 1;
                    } else {
                        app.selected_molecular_tab = app.molecular_tabs.len().saturating_sub(1);
                    }
                },
                7 => {
                    if !app.molecular_tabs.is_empty() {
                        app.selected_molecular_tab = (app.selected_molecular_tab + 1) % app.molecular_tabs.len();
                    }
                },
                _ => app.mark_dirty(),
            }

            assert!(app.selected_tab < app.tabs.len());
        }
    }
}
