//! Gravador de PNG mínimo (RGBA 8 bits, sem compressão) para a captura de tela do modo mock.
//! Sem dependências: o zlib usa blocos "stored", o PNG fica grande mas é só para conferência.

fn crc32(dados: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &b in dados {
        crc ^= u32::from(b);
        for _ in 0..8 {
            crc = if crc & 1 != 0 { (crc >> 1) ^ 0xEDB8_8320 } else { crc >> 1 };
        }
    }
    !crc
}

fn adler32(dados: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    for &x in dados {
        a = (a + u32::from(x)) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}

fn bloco(saida: &mut Vec<u8>, tipo: &[u8; 4], dados: &[u8]) {
    saida.extend_from_slice(&(dados.len() as u32).to_be_bytes());
    let mut corpo = tipo.to_vec();
    corpo.extend_from_slice(dados);
    saida.extend_from_slice(&corpo);
    saida.extend_from_slice(&crc32(&corpo).to_be_bytes());
}

/// Codifica `rgba` (largura x altura x 4 bytes) como PNG.
pub fn encode_rgba(largura: u32, altura: u32, rgba: &[u8]) -> Vec<u8> {
    assert_eq!(rgba.len(), largura as usize * altura as usize * 4, "tamanho do buffer");
    // Cada linha começa com o byte de filtro 0.
    let mut cru = Vec::with_capacity(rgba.len() + altura as usize);
    for linha in rgba.chunks(largura as usize * 4) {
        cru.push(0);
        cru.extend_from_slice(linha);
    }
    // zlib: cabeçalho, blocos "stored" de até 65535 bytes, soma adler32.
    let mut z = vec![0x78, 0x01];
    let mut blocos = cru.chunks(65535).peekable();
    if blocos.peek().is_none() {
        z.extend_from_slice(&[1, 0, 0, 0xFF, 0xFF]);
    }
    while let Some(b) = blocos.next() {
        z.push(u8::from(blocos.peek().is_none()));
        z.extend_from_slice(&(b.len() as u16).to_le_bytes());
        z.extend_from_slice(&(!(b.len() as u16)).to_le_bytes());
        z.extend_from_slice(b);
    }
    z.extend_from_slice(&adler32(&cru).to_be_bytes());

    let mut saida = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&largura.to_be_bytes());
    ihdr.extend_from_slice(&altura.to_be_bytes());
    ihdr.extend_from_slice(&[8, 6, 0, 0, 0]); // 8 bits, RGBA, sem entrelaçamento
    bloco(&mut saida, b"IHDR", &ihdr);
    bloco(&mut saida, b"IDAT", &z);
    bloco(&mut saida, b"IEND", &[]);
    saida
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc_e_adler_conhecidos() {
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
        assert_eq!(adler32(b"Wikipedia"), 0x11E6_0398);
    }

    #[test]
    fn png_tem_assinatura_blocos_validos_e_pixels_de_volta() {
        // Imagem maior que um bloco "stored" (65535) para exercitar a divisão.
        let (l, a) = (200u32, 120u32);
        let rgba: Vec<u8> = (0..l * a * 4).map(|i| (i % 251) as u8).collect();
        let png = encode_rgba(l, a, &rgba);
        assert_eq!(&png[..8], &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]);

        let mut pos = 8;
        let mut idat = Vec::new();
        let mut tipos = Vec::new();
        while pos < png.len() {
            let n = u32::from_be_bytes(png[pos..pos + 4].try_into().unwrap()) as usize;
            let corpo = &png[pos + 4..pos + 8 + n];
            let crc = u32::from_be_bytes(png[pos + 8 + n..pos + 12 + n].try_into().unwrap());
            assert_eq!(crc32(corpo), crc, "crc do bloco");
            tipos.push(String::from_utf8_lossy(&corpo[..4]).to_string());
            if &corpo[..4] == b"IDAT" {
                idat.extend_from_slice(&corpo[4..]);
            }
            if &corpo[..4] == b"IHDR" {
                assert_eq!(u32::from_be_bytes(corpo[4..8].try_into().unwrap()), l);
                assert_eq!(u32::from_be_bytes(corpo[8..12].try_into().unwrap()), a);
            }
            pos += 12 + n;
        }
        assert_eq!(tipos, ["IHDR", "IDAT", "IEND"]);

        // Desfaz o zlib "stored" e confere os pixels.
        let mut dados = &idat[2..idat.len() - 4];
        let mut cru = Vec::new();
        loop {
            let ultimo = dados[0] & 1 == 1;
            let n = u16::from_le_bytes([dados[1], dados[2]]) as usize;
            assert_eq!(u16::from_le_bytes([dados[3], dados[4]]), !(n as u16));
            cru.extend_from_slice(&dados[5..5 + n]);
            dados = &dados[5 + n..];
            if ultimo {
                break;
            }
        }
        assert_eq!(adler32(&cru).to_be_bytes(), idat[idat.len() - 4..]);
        let esperado: Vec<u8> = rgba
            .chunks(l as usize * 4)
            .flat_map(|linha| std::iter::once(0u8).chain(linha.iter().copied()))
            .collect();
        assert_eq!(cru, esperado);
    }
}
