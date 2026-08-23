import { useState, useRef, useEffect } from 'react';
import { Button } from '../common/Button';
import type { Family } from '../../types/domain';
import { formatDate } from '../../utils/dateUtils';

export interface FamilyCardProps {
  family: Family;
  onEdit: (family: Family) => void;
  onDelete: (family: Family) => void;
  onView: (family: Family) => void;
  onRecipes: (family: Family) => void;
}

/**
 * FamilyCard component displays a single family with action buttons
 * - Displays family name and timestamps (Requirement 3.3, 3.4)
 * - Provides edit and delete actions (Requirements 4.1, 5.1)
 */
export function FamilyCard({ family, onEdit, onDelete, onView, onRecipes }: FamilyCardProps) {
  const [menuOpen, setMenuOpen] = useState(false);
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!menuOpen) return;
    const handler = (e: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        setMenuOpen(false);
      }
    };
    document.addEventListener('mousedown', handler);
    return () => document.removeEventListener('mousedown', handler);
  }, [menuOpen]);

  return (
    <div className="mb-3 p-4 bg-white rounded-xl border border-gray-100 shadow-sm">
      <div className="flex items-start justify-between">
        <h3 className="text-base font-semibold mb-2 flex-1 min-w-0 truncate">{family.name}</h3>
        <div className="relative" ref={menuRef}>
          <button
            onClick={() => setMenuOpen(!menuOpen)}
            className="p-1 text-gray-400 hover:text-gray-600 hover:bg-gray-100 rounded-lg transition-colors"
          >
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M12 5v.01M12 12v.01M12 19v.01"
              />
            </svg>
          </button>
          {menuOpen && (
            <div className="absolute right-0 z-10 mt-1 w-44 bg-white rounded-xl shadow-lg border border-gray-100 py-1">
              <button
                onClick={() => {
                  onRecipes(family);
                  setMenuOpen(false);
                }}
                className="w-full text-left px-3 py-2 text-sm text-gray-700 hover:bg-gray-50 flex items-center gap-2"
              >
                <span className="text-base">🍽️</span> Recipes
              </button>
              <button
                onClick={() => {
                  onView(family);
                  setMenuOpen(false);
                }}
                className="w-full text-left px-3 py-2 text-sm text-gray-700 hover:bg-gray-50 flex items-center gap-2"
              >
                <span className="text-base">👁️</span> View details
              </button>
              <button
                onClick={() => {
                  onEdit(family);
                  setMenuOpen(false);
                }}
                className="w-full text-left px-3 py-2 text-sm text-gray-700 hover:bg-gray-50 flex items-center gap-2"
              >
                <span className="text-base">✏️</span> Edit
              </button>
              <div className="border-t border-gray-100 my-1" />
              <button
                onClick={() => {
                  onDelete(family);
                  setMenuOpen(false);
                }}
                className="w-full text-left px-3 py-2 text-sm text-red-600 hover:bg-red-50 flex items-center gap-2"
              >
                <span className="text-base">🗑️</span> Delete
              </button>
            </div>
          )}
        </div>
      </div>
      <div className="text-sm text-muted">
        Created: {formatDate(family.created_at)}
        <br />
        Updated: {formatDate(family.updated_at)}
      </div>
      <div className="flex gap-2 mt-3">
        <Button variant="primary" size="sm" icon="eye" onClick={() => onView(family)} />
        <Button variant="secondary" size="sm" icon="pencil" onClick={() => onEdit(family)} />
        <Button variant="danger" size="sm" icon="trash" onClick={() => onDelete(family)} />
      </div>
    </div>
  );
}
