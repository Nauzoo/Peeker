<script>
    import { onMount } from "svelte";

    import Login from "./pages/Login.svelte";
    import Register from "./pages/Register.svelte";
    import Dashboard from "./pages/Dashboard.svelte";
    import { appState } from "./GlobalState.svelte.js";

    const page = {
        loading: "loading",
        login: "login",
        register: "register",
        dashboard: "dashboard",
    };

    let currentScreen = $state(page.loading);

    function goToDashboard() {
        currentScreen = page.dashboard;
    }
    function goToRegister() {
        currentScreen = page.register;
    }
    function goToLogin() {
        currentScreen = page.login;
    }

    // onMount automatically executes whenever the page first loads
    onMount(async () => {
        try {
            const response = await fetch("/api/auth/", {
                method: "GET",
                credentials: "include",
                headers: { "Content-Type": "application/json" },
            });

            if (response.ok) {
                const data = await response.json();
                appState.user_id = data.id;
                appState.user_roll = data.role;

                goToDashboard();
            } else {
                goToLogin();
            }
        } catch (erro) {
            console.error("Erro ao verificar o servidor:", erro);
            goToLogin(); // Se der erro de rede, joga pro login por segurança
        }
    });
</script>

<main>
    {#if currentScreen === page.loading}
        <p style="text-align: center; margin-top: 50px;">
            Verificando sessão...
        </p>
    {:else if currentScreen === page.login}
        <Login irParaDashboard={goToDashboard} irParaRegistro={goToRegister} />
    {:else if currentScreen === page.register}
        <Register irParaLogin={goToLogin} />
    {:else if currentScreen === page.dashboard}
        <Dashboard />
    {/if}
</main>
