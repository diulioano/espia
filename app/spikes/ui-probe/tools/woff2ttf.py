"""Converte WOFF (versão 1) em TTF: só descomprime as tabelas (zlib) e remonta o cabeçalho sfnt.
Uso: python -I tools/woff2ttf.py entrada.woff saida.ttf
A Slint só carrega TTF/OTF; o pacote @fontsource traz apenas woff/woff2. Sem dependências."""
import struct, sys, zlib

def converter(entrada, saida):
    d = open(entrada, 'rb').read()
    assinatura, flavor, _len, n, _res, _tam, *_ = struct.unpack('>4sIIHHI', d[:20])
    assert assinatura == b'wOFF', 'não é WOFF 1'
    tabelas = []
    for i in range(n):
        tag, off, comp, orig, csum = struct.unpack('>4sIIII', d[44 + 20 * i: 64 + 20 * i])
        bruto = d[off:off + comp]
        dados = zlib.decompress(bruto) if comp != orig else bruto
        assert len(dados) == orig
        tabelas.append((tag, csum, dados))
    tabelas.sort(key=lambda t: t[0])
    pot = 1
    while pot * 2 <= n:
        pot *= 2
    cab = struct.pack('>IHHHH', flavor, n, pot * 16, pot.bit_length() - 1, n * 16 - pot * 16)
    pos = 12 + 16 * n
    dir_, corpo = b'', b''
    for tag, csum, dados in tabelas:
        dir_ += struct.pack('>4sIII', tag, csum, pos + len(corpo), len(dados))
        corpo += dados + b'\0' * (-len(dados) % 4)
    open(saida, 'wb').write(cab + dir_ + corpo)

converter(sys.argv[1], sys.argv[2])
