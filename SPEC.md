# Especificação da Linguagem Nova (rascunho v0.1)

## Visão

Nova é uma linguagem de programação ultra-simples, full-stack e de alta produtividade.
Foi desenhada para ser **significativamente mais fácil de aprender e usar do que Python**, mantendo poder suficiente para:

- Treinar modelos de IA
- Criar sites e aplicações web
- Programas desktop
- Scripts e automação
- Qualquer outra coisa

## Princípios de design

1. **Mínimo de palavras-chave e símbolos**
2. **Zero boilerplate** — o menos código possível para fazer coisas úteis
3. **Uma única linguagem** para frontend, backend, AI e scripts
4. **Legibilidade extrema**
5. **Tipagem opcional e inteligente**

## Extensões de ficheiro

- `.nv`
- `.nova`

## Sintaxe básica (proposta)

### Comentários
```nova
# isto é um comentário
```

### Variáveis
Não é necessário `let`, `var` ou `const`.
```nova
nome = "Nova"
idade = 1
ativo = verdadeiro
```

### Saída
```nova
diz "Olá, mundo!"
diz nome
```

### Funções
```nova
fun somar(a, b)
  retorna a + b
fim
```

### Condicionais
```nova
se idade > 18
  diz "Adulto"
senão
  diz "Menor"
fim
```

### Loops
```nova
para i de 1 a 10
  diz i
fim

enquanto verdadeiro
  diz "loop"
  parar
fim
```

### Estruturas de dados
```nova
lista = [1, 2, 3]
mapa = { nome: "Nova", versao: 0.1 }
```

## Próximos passos

- Definir palavras-chave finais (português vs inglês)
- Sistema de tipos
- Módulos e importação
- Integração com AI / treino de modelos
- Runtime / interpretador

---

Este é um rascunho vivo. Tudo pode mudar.