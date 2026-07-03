<script lang="js">
    let username = "";
    let password = "";
    let role = "";

    function postLogin(){
        fetch("/register", {
            method: 'POST',
            credentials: 'include',
            headers: {
                'Content-type': 'application/json'
            },
            body: JSON.stringify({ username, password, role })
        }).then(response => {
            if (!response.ok) 
                throw new Error("Login Failed" + response.status);
            return response;
        }).then(data => {
            console.log(data);
        }).catch(error => {
            console.error("There was an applicatin error" + error);
        })
    }
</script>

<div class="login-container">
    <form on:submit|preventDefault={postLogin}>
        <h2>Acesse sua conta</h2>

        <div class="input-group">
            <label for="username">Usuário</label>
            <input id="username" type="text" bind:value={username} required placeholder="Digite seu usuário">
        </div>

        <div class="input-group">
            <label for="password">Senha</label>
            <input id="password" type="password" bind:value={password} required placeholder="Digite sua senha">
        </div>

        
        <div class="input-group">
            <label for="role">Cargo</label>
            <input id="role" type="text" bind:value={role} required placeholder="Digite seu cargo">
        </div>

        <button type="submit">Registrar</button>
    </form>
</div>

<style>
    /* 1. O Contêiner Principal (Centralização) */
    .login-container {
        display: flex; /* Habilita o Flexbox */
        justify-content: center; /* Centraliza horizontalmente */
        align-items: center; /* Centraliza verticalmente */
        height: 100vh; /* Ocupa 100% da altura da tela (Viewport Height) */
        background-color: #f3f4f6; /* Uma cor de fundo cinza bem clara */
        font-family: sans-serif; /* Fonte limpa e moderna */
    }

    /* 2. O Cartão do Formulário */
    form {
        display: flex;
        flex-direction: column; /* Coloca os elementos um embaixo do outro */
        background-color: white;
        padding: 2rem; /* Espaçamento interno (32px) */
        border-radius: 8px; /* Bordas arredondadas */
        box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1); /* Sombra suave para dar profundidade */
        width: 100%;
        max-width: 350px; /* Limita a largura para não ficar gigante em telas grandes */
    }

    h2 {
        text-align: center;
        color: #333;
        margin-top: 0;
        margin-bottom: 1.5rem;
    }

    /* 3. Agrupamento de Inputs */
    .input-group {
        display: flex;
        flex-direction: column; /* Label em cima, Input embaixo */
        margin-bottom: 1rem; /* Espaço entre os campos */
    }

    label {
        font-size: 0.875rem;
        color: #555;
        margin-bottom: 0.25rem;
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