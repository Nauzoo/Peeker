<script lang="ts">
    import { onMount } from "svelte";
    import {
        LightboxGallery,
        GalleryImage,
        GalleryThumbnail,
    } from "svelte-lightbox";

    import { appState } from "../GlobalState.svelte.js";

    interface file {
        id: string | number;
        name: string;
    }

    function isVideo(filename: string): boolean {
        if (!filename) return false;
        const videoExtensions = [
            ".mp4",
            ".webm",
            ".ogg",
            ".mov",
            ".mkv",
            ".avi",
        ];
        return videoExtensions.some((ext) =>
            filename.toLowerCase().endsWith(ext),
        );
    }

    let files: file[] = $state([]);
    let currentPage = $state(1);
    let isLoading = $state(false);
    let hasMoreFiles = $state(true);

    let batchAmount = 20;

    let listEndWatcher: Element;

    async function fetchFiles() {
        if (isLoading || !hasMoreFiles) return;

        isLoading = true;

        try {
            const response = await fetch(
                `/api/files?page=${currentPage}&amount=${batchAmount}`,
                {
                    credentials: "include", // <- needed to verify the browser's credentials
                },
            );

            if (!response.ok) {
                console.error("A API retornou um erro:", response.status);
                hasMoreFiles = false; // Para de tentar buscar
                return; // Aborta a função antes de quebrar o Javascript
            }

            const fileBatch = await response.json();

            if (fileBatch.length === 0) {
                // empty files list -> no more files
                hasMoreFiles = false;
            } else {
                files = [...files, ...fileBatch]; // Self note: This weird notations means files += filesBatch
                // Svelte's reactivity needs an attribution in order to update the state of this objecct
                currentPage += 1;
            }
        } catch (erro) {
            console.error("Erro ao carregar arquivos:", erro);
        } finally {
            isLoading = false;
        }
    }

    // Touch swipe gesture handling for mobile lightbox navigation
    let touchStartX = 0;
    let touchStartY = 0;
    let touchEndX = 0;
    let touchEndY = 0;
    const minSwipeDistance = 40;

    function handleTouchStart(event: TouchEvent) {
        if (event.touches.length !== 1) return;
        if (!document.querySelector(".svelte-lightbox-overlay")) return;

        touchStartX = event.touches[0].clientX;
        touchStartY = event.touches[0].clientY;
    }

    function handleTouchEnd(event: TouchEvent) {
        if (!document.querySelector(".svelte-lightbox-overlay")) return;
        if (event.changedTouches.length !== 1) return;

        touchEndX = event.changedTouches[0].clientX;
        touchEndY = event.changedTouches[0].clientY;

        const deltaX = touchEndX - touchStartX;
        const deltaY = touchEndY - touchStartY;

        // Ensure horizontal swipe distance is larger than vertical movement
        if (
            Math.abs(deltaX) > minSwipeDistance &&
            Math.abs(deltaX) > Math.abs(deltaY)
        ) {
            if (deltaX < 0) {
                // Swipe left -> Next item
                const nextBtn =
                    document.querySelector<HTMLButtonElement>(".next-button");
                if (nextBtn && !nextBtn.disabled) {
                    nextBtn.click();
                }
            } else {
                // Swipe right -> Previous item
                const prevBtn =
                    document.querySelector<HTMLButtonElement>(
                        ".previous-button",
                    );
                if (prevBtn && !prevBtn.disabled) {
                    prevBtn.click();
                }
            }
        }
    }

    onMount(() => {
        const observer = new IntersectionObserver((entries) => {
            const watcher = entries[0];
            // isIntersecting is true when the div enters the browser's viewport
            if (watcher.isIntersecting) {
                fetchFiles();
            }
        });

        if (listEndWatcher) {
            observer.observe(listEndWatcher);
        }

        window.addEventListener("touchstart", handleTouchStart, {
            passive: true,
        });
        window.addEventListener("touchend", handleTouchEnd, { passive: true });

        // Cleans the memory when the user leaves the page
        return () => {
            observer.disconnect();
            window.removeEventListener("touchstart", handleTouchStart);
            window.removeEventListener("touchend", handleTouchEnd);
        };
    });

    let arquivoSelecionado = $state<File | null | undefined>(null);
    let mensagem = $state("");

    function aoSelecionarArquivo(evento: Event) {
        const elemento = evento.target as HTMLInputElement;
        arquivoSelecionado = elemento.files?.[0];
    }

    async function fazerUpload() {
        if (!arquivoSelecionado) {
            mensagem = "Por favor, selecione um arquivo primeiro.";
            return;
        }

        mensagem = "Enviando...";

        // 1. Prepara o formato Multipart
        const formData = new FormData();
        // O nome "arquivo" aqui não importa muito para o nosso backend,
        // mas é obrigatório dar um nome ao campo.
        formData.append("arquivo", arquivoSelecionado);

        try {
            // 2. Faz a requisição
            const resposta = await fetch("/api/upload", {
                method: "POST",
                credentials: "include", // <-- ESSENCIAL para enviar o cookie de login!
                body: formData, // Quando passamos FormData, o navegador ajusta os Headers automaticamente
            });

            if (resposta.ok) {
                const dados = await resposta.json();
                mensagem = `Sucesso! Arquivo salvo como: ${dados.id}`;
                console.log(dados);
            } else {
                mensagem = `Erro no upload: ${resposta.status}`;
            }
        } catch (erro) {
            console.error(erro);
            mensagem = "Erro de conexão.";
        }
    }
</script>

<!--
Dash board Containing the file grid and its' functionalities 
-->

<main>
    <h2>Meus Arquivos</h2>

    <div class="flex justify-center gap-4">
        <p>Você está logado como: {appState.user_roll}</p>
    </div>

    <div style="padding: 20px; border: 1px solid #ccc; max-width: 400px;">
        <h3>Testar Upload</h3>

        <input type="file" onchange={aoSelecionarArquivo} />

        <button onclick={fazerUpload} style="margin-top: 10px; display: block;">
            Enviar Arquivo
        </button>

        {#if mensagem}
            <p style="margin-top: 15px; font-weight: bold;">{mensagem}</p>
        {/if}
    </div>

    <!-- code for a basic reactive masonry grid wrapped in LightboxGallery -->
    <LightboxGallery
        arrowsConfig={{
            color: "white",
            character: "",
            enableKeyboardControl: true,
        }}
    >
        <div
            slot="thumbnail"
            class="columns-2 sm:columns-2 lg:columns-3 py-10 md:py-20 gap-4"
        >
            {#each files as a_file, index (a_file.id)}
                <div class="mb-4 break-inside-avoid">
                    <GalleryThumbnail id={index}>
                        {#if isVideo(a_file.name)}
                            <video
                                class="w-full object-cover rounded-lg pointer-events-none"
                                src={`/api/files/test_files/${a_file.id}`}
                                autoplay
                                muted
                                playsinline
                            >
                                <track kind="captions" />
                            </video>
                        {:else}
                            <img
                                class="w-full object-cover rounded-lg"
                                src={`/api/files/test_files/${a_file.id}`}
                                alt={a_file.name}
                            />
                        {/if}
                    </GalleryThumbnail>
                </div>
            {/each}
        </div>

        {#each files as a_file (a_file.id)}
            <GalleryImage title={a_file.name}>
                {#if isVideo(a_file.name)}
                    <video
                        class="max-h-[80vh] max-w-full rounded-lg"
                        controls
                        autoplay
                        muted
                        src={`/api/files/test_files/${a_file.id}`}
                    >
                        <track kind="captions" />
                    </video>
                {:else}
                    <img
                        class="max-h-[80vh] max-w-full object-contain rounded-lg"
                        src={`/api/files/test_files/${a_file.id}`}
                        alt={a_file.name}
                    />
                {/if}
            </GalleryImage>
        {/each}
    </LightboxGallery>

    <div bind:this={listEndWatcher} class="watcher">
        {#if isLoading}
            <p>Carregando mais arquivos...</p>
        {:else if !hasMoreFiles}
            <p>Você chegou ao fim! Não há mais arquivos.</p>
        {/if}
    </div>
</main>

<style>
    @import "tailwindcss";

    .watcher {
        text-align: center;
        padding: 2rem;
        margin-top: 1rem;

        min-height: 50px;
    }

    :global(.previous-button),
    :global(.next-button) {
        position: fixed !important;
        top: 50% !important;
        transform: translateY(-50%) !important;
        width: 3.25rem !important;
        height: 3.25rem !important;
        bottom: auto !important;
        z-index: 1000005 !important;
        display: flex !important;
        align-items: center !important;
        justify-content: center !important;
        background-color: rgba(0, 0, 0, 0.6) !important;
        backdrop-filter: blur(4px) !important;
        border-radius: 9999px !important;
        color: white !important;
        cursor: pointer !important;
        transition:
            background-color 0.2s ease,
            transform 0.2s ease !important;
        padding: 0 !important;
    }

    :global(.previous-button) {
        left: 1.5rem !important;
        right: auto !important;
    }

    :global(.next-button) {
        right: 1.5rem !important;
        left: auto !important;
    }

    :global(.previous-button:hover:not(:disabled)),
    :global(.next-button:hover:not(:disabled)) {
        background-color: rgba(0, 0, 0, 0.9) !important;
        transform: translateY(-50%) scale(1.1) !important;
    }

    :global(.previous-button:disabled),
    :global(.next-button:disabled) {
        opacity: 0.3 !important;
        cursor: not-allowed !important;
    }

    :global(.previous-button svg),
    :global(.next-button svg) {
        height: 1.75rem !important;
        width: 1.75rem !important;
    }

    :global(.previous-button .arrow),
    :global(.next-button .arrow) {
        stroke: #ffffff !important;
    }

    /* Hide navigation arrows on mobile screens (swipe gestures are used instead) */
    @media (max-width: 768px) {
        :global(.previous-button),
        :global(.next-button) {
            display: none !important;
            visibility: hidden !important;
            pointer-events: none !important;
        }
    }
</style>
