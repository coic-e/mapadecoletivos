import { Link } from "react-router-dom";
import { BottomCenter, Button, Container, TopCenter } from "./styles";
import { FiArrowRight } from "react-icons/fi";

export default function Overlay() {
  return (
    <Container>
      <TopCenter>
        <h1>MAPA DE RAVE</h1>
        <p>Descubra a Batida do Underdroung</p>
        <p>
          Seja bem-vindo à nossa família rave! No Mapa de Rave, você não apenas encontra festas;
        </p>
        <p>
          Você descobre portais para experiências inesquecíveis onde a batida da
          música eletrônica pulsa em harmonia com seu espírito aventureiro.
        </p>
        <Button>
          <Link to="/raves">
            <FiArrowRight size={26} color="rgba(0, 0, 0, 0.6)" />
          </Link>
        </Button>
      </TopCenter>

      <BottomCenter>
        <span>
          Desenvolvido por <a href="">COICE PRODUÇÕES</a>
        </span>
        <span>
          Facebook <a href="https://www.facebook.com/coic.e">Facebook</a>
        </span>
        <span>
          Instagram <a href="https://www.instagram.com/coic_e/">@coic_e</a>
        </span>
        <span>
          Soundcloud <a href="https://soundcloud.com/coic_e">Soundcloud</a>
        </span>
      </BottomCenter>
    </Container>
  );
}
