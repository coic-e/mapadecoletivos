-- Seed data for organizations (Brazilian electronic music collectives)

-- Clear existing data
DELETE FROM images;
DELETE FROM organizations;

-- Reset sequences
ALTER SEQUENCE organizations_id_seq RESTART WITH 1;
ALTER SEQUENCE images_id_seq RESTART WITH 1;

-- Insert organizations
INSERT INTO organizations (name, latitude, longitude, type, city, uf, email, social, about) VALUES
(
    'Mamba Negra',
    -23.55,
    -46.63,
    'Coletivo',
    'São Paulo',
    'SP',
    'contato@mambanegrasp.com',
    '@mambanegrasp',
    'Coletivo paulistano dedicado à música techno e house, promovendo festas com DJs locais e internacionais desde 2015.'
),
(
    'VOODOOHOP',
    -23.54,
    -46.64,
    'Coletivo',
    'São Paulo',
    'SP',
    'info@voodoohop.com',
    '@voodoohop',
    'Plataforma global de música eletrônica experimental com sede em São Paulo, conectando artistas de todo o mundo através de festas, releases e residências artísticas.'
),
(
    'Beijo',
    -22.91,
    -43.18,
    'Festa',
    'Rio de Janeiro',
    'RJ',
    'hello@beijo.party',
    '@beijoparty',
    'Festa LGBTQIA+ carioca focada em house, disco e vogue, criando espaços seguros e inclusivos na cena eletrônica desde 2018.'
),
(
    'Selvagem',
    -22.95,
    -43.20,
    'Coletivo',
    'Rio de Janeiro',
    'RJ',
    'contato@selvagem.rio',
    '@selvagemrj',
    'Coletivo carioca multi-disciplinar que une música eletrônica, arte visual e performance em festas imersivas.'
),
(
    'Gop Tun',
    -15.79,
    -47.89,
    'Coletivo',
    'Brasília',
    'DF',
    'goptun@goptun.com',
    '@goptun',
    'Coletivo brasiliense pioneiro na cena de música eletrônica do cerrado, promovendo techno, trance e música experimental desde 2005.'
),
(
    'Trottoir',
    -19.92,
    -43.93,
    'Festa',
    'Belo Horizonte',
    'MG',
    'contato@trottoir.com.br',
    '@trottoir_bh',
    'Festa mineira underground dedicada ao techno, trance e breaks, realizando eventos em locais alternativos de BH.'
),
(
    'Modo Avião',
    -12.97,
    -38.51,
    'Coletivo',
    'Salvador',
    'BA',
    'modo@aviaocoletivo.com',
    '@modoaviao',
    'Coletivo baiano que mescla música eletrônica com ritmos afro-brasileiros, criando uma sonoridade única e tropical.'
),
(
    'Playground Records',
    -3.73,
    -38.52,
    'Label',
    'Fortaleza',
    'CE',
    'info@playgroundrecords.com',
    '@playgroundrecordsce',
    'Selo e coletivo cearense especializado em house, disco e música experimental, desenvolvendo a cena local desde 2016.'
),
(
    'Agora',
    -8.05,
    -34.90,
    'Coletivo',
    'Recife',
    'PE',
    'agora@agorarecife.com',
    '@agora.recife',
    'Coletivo pernambucano que promove a cultura eletrônica através de festas, workshops e apoio a artistas locais.'
),
(
    'Kaos',
    -25.43,
    -49.27,
    'Coletivo',
    'Curitiba',
    'PR',
    'kaos@kaoscuritiba.com',
    '@kaos_cwb',
    'Coletivo curitibano focado em techno industrial e EBM, trazendo a sonoridade dark para o sul do Brasil.'
),
(
    'Techno Tuga',
    -30.03,
    -51.22,
    'Festa',
    'Porto Alegre',
    'RS',
    'contato@technotuga.com',
    '@technotuga_poa',
    'Festa gaúcha de techno que acontece mensalmente, reunindo a comunidade eletrônica do sul do país.'
),
(
    'Grooveteria',
    -23.56,
    -46.65,
    'Loja/Coletivo',
    'São Paulo',
    'SP',
    'info@grooveteria.com.br',
    '@grooveteria',
    'Loja de discos e coletivo paulistano especializado em house, disco e funk, promovendo também festas e eventos musicais.'
);

-- Note: Images will be added through the API's multipart upload endpoint
-- This seed data focuses on organizations only
