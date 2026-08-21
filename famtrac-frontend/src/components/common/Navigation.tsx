import { Link, useNavigate } from 'react-router-dom';
import { useState } from 'react';
import { useAuth } from '../../auth/useAuth';
import { Button } from './Button';

/**
 * Navigation component - Main navigation bar
 * - Displays navigation links (Requirement 14.1)
 * - Shows current user information (Requirement 1.6)
 * - Includes logout button (Requirement 1.6)
 */
export function Navigation() {
  const { user, logout, isAuthenticated } = useAuth();
  const navigate = useNavigate();
  const [menuOpen, setMenuOpen] = useState(false);

  const handleLogout = async () => {
    await logout();
    navigate('/');
  };

  // Don't show navigation if not authenticated
  if (!isAuthenticated) {
    return null;
  }

  return (
    <nav className="sticky top-0 z-50 bg-blue-600 shadow-sm">
      <div className="max-w-5xl mx-auto px-4">
        <div className="flex items-center justify-between h-14">
          {/* Brand */}
          <Link to="/" className="text-white text-lg font-semibold">
            FamTrac
          </Link>

          {/* Desktop nav */}
          <div className="hidden md:flex items-center gap-6">
            <div className="flex items-center gap-6">
              <Link to="/" className="text-white/80 hover:text-white text-sm font-medium transition-colors">
                Families
              </Link>
              <Link to="/shares" className="text-white/80 hover:text-white text-sm font-medium transition-colors">
                Shared With Me
              </Link>
            </div>
            <div className="flex items-center gap-3 ml-4 pl-4 border-l border-white/20">
              {user && (
                <span className="text-white/80 text-sm">
                  Signed in as: <strong className="text-white">{String(user.username || user.sub)}</strong>
                </span>
              )}
              <Button variant="link" size="sm" onClick={handleLogout} className="text-white/80 hover:text-white">
                Logout
              </Button>
            </div>
          </div>

          {/* Mobile menu toggle */}
          <button
            onClick={() => setMenuOpen(!menuOpen)}
            className="md:hidden text-white p-2"
            aria-label="Toggle menu"
            aria-expanded={menuOpen}
          >
            <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              {menuOpen ? (
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
              ) : (
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 12h16M4 18h16" />
              )}
            </svg>
          </button>
        </div>

        {/* Mobile menu */}
        {menuOpen && (
          <>
            {/* Backdrop */}
            <div
              className="fixed inset-0 bg-black/40 z-40 md:hidden"
              onClick={() => setMenuOpen(false)}
              aria-hidden="true"
            />
            {/* Menu panel */}
            <div className="fixed top-0 right-0 h-full w-64 bg-blue-700 z-50 md:hidden shadow-lg">
              <div className="flex items-center justify-between h-14 px-4 border-b border-white/10">
                <Link to="/" className="text-white text-lg font-semibold" onClick={() => setMenuOpen(false)}>
                  FamTrac
                </Link>
                <button
                  onClick={() => setMenuOpen(false)}
                  className="text-white/80 hover:text-white p-2"
                  aria-label="Close menu"
                >
                  <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                  </svg>
                </button>
              </div>
              <div className="flex flex-col gap-1 p-4">
                <Link
                  to="/"
                  className="text-white/80 hover:text-white text-sm font-medium transition-colors px-3 py-2 rounded-lg hover:bg-white/10"
                  onClick={() => setMenuOpen(false)}
                >
                  Families
                </Link>
                <Link
                  to="/shares"
                  className="text-white/80 hover:text-white text-sm font-medium transition-colors px-3 py-2 rounded-lg hover:bg-white/10"
                  onClick={() => setMenuOpen(false)}
                >
                  Shared With Me
                </Link>
                {user && (
                  <span className="text-white/60 text-xs px-3 pt-2">
                    Signed in as: <strong className="text-white/80">{String(user.username || user.sub)}</strong>
                  </span>
                )}
                <div className="px-3 pt-2">
                  <Button variant="link" size="sm" onClick={handleLogout} className="text-white/80 hover:text-white">
                    Logout
                  </Button>
                </div>
              </div>
            </div>
          </>
        )}
      </div>
    </nav>
  );
}
