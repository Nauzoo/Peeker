<script>
    // Recebemos as funções do App.svelte
    /** @type {() => void} */
    export let irParaLogin;

    let username = "";
    let password = "";
    let role = ""

    async function processarRegistro() {
        const resposta = await fetch("/api/register", {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ username, password, role })
        });

        if (resposta.ok) {
            console.log("Login de sucesso! Trocando de tela...");
            irParaLogin(); // <--- Troca a tela na hora!
        } else {
            alert("Usuário ou senha incorretos.");
        }
    }
</script>

<h2>Login</h2>
<input type="text" bind:value={username} />
<input type="password" bind:value={password} />
<input type="cargo" bind:value={role} />
<button on:click={processarRegistro}>Entrar</button>

<style>

    h2 {
        text-align: center;
        color: #333;
        margin-top: 0;
        margin-bottom: 1.5rem;
    }
    /* 4. Estilo das Caixas de Texto */
    input {
        padding: 0.75rem;
        border: 1px solid #ccc;
        border-radius: 4px;
        font-size: 1rem;
        transition: border-color 0.3s; /* Animação suave ao focar */
    }

    /* Quando o usuário clica no input */
    input:focus {
        outline: none;
        border-color: #007bff; /* Borda azul ao digitar */
    }

    /* 5. Estilo do Botão */
    button {
        margin-top: 1rem;
        padding: 0.75rem;
        background-color: #007bff; /* Azul clássico */
        color: white;
        border: none;
        border-radius: 4px;
        font-size: 1rem;
        font-weight: bold;
        cursor: pointer; /* Muda o mouse para a "mãozinha" */
        transition: background-color 0.3s;
    }

    /* Efeito ao passar o mouse por cima do botão */
    button:hover {
        background-color: #0056b3; /* Azul um pouco mais escuro */
    }
</style>