import { Navbar, Container, Nav, Button } from 'react-bootstrap';
import { Link, useNavigate } from 'react-router-dom';
import { useAuth } from '../../auth/useAuth';

/**
 * Navigation component - Main navigation bar
 * - Displays navigation links (Requirement 14.1)
 * - Shows current user information (Requirement 1.6)
 * - Includes logout button (Requirement 1.6)
 */
export function Navigation() {
  const { user, logout, isAuthenticated } = useAuth();
  const navigate = useNavigate();

  const handleLogout = async () => {
    await logout();
    navigate('/');
  };

  // Don't show navigation if not authenticated
  if (!isAuthenticated) {
    return null;
  }

  return (
    <Navbar bg="primary" variant="dark" expand="lg" className="mb-4">
      <Container>
        <Navbar.Brand as={Link} to="/">
          FamTrac
        </Navbar.Brand>
        <Navbar.Toggle aria-controls="basic-navbar-nav" />
        <Navbar.Collapse id="basic-navbar-nav">
          <Nav className="me-auto">
            <Nav.Link as={Link} to="/">
              Families
            </Nav.Link>
            <Nav.Link as={Link} to="/shares">
              Shared With Me
            </Nav.Link>
          </Nav>
          <Nav className="ms-auto align-items-center">
            {user && (
              <Navbar.Text className="me-3">
                Signed in as: <strong>{String(user.email || user.sub)}</strong>
              </Navbar.Text>
            )}
            <Button variant="outline-light" size="sm" onClick={handleLogout}>
              Logout
            </Button>
          </Nav>
        </Navbar.Collapse>
      </Container>
    </Navbar>
  );
}
