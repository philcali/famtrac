export function formatDate(date: string | Date, options?: Intl.DateTimeFormatOptions): string {
  let d: Date;
  if (typeof date === 'string') {
    d = new Date(date);
    if (!date.match(/T/)) {
      d.setTime(d.getTime() + d.getTimezoneOffset() * 60 * 1000);
    }
  } else {
    d = date;
  }
  return d.toLocaleDateString(
    'en-US',
    options ?? {
      year: 'numeric',
      month: 'long',
      day: 'numeric',
    }
  );
}

export function formatDateTime(date: string | Date): string {
  const d = typeof date === 'string' ? new Date(date) : date;
  return d.toLocaleString('en-US', {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
}

export function formatTime(dateString: string | Date) {
  return new Date(dateString).toLocaleTimeString([], {
    hour: '2-digit',
    minute: '2-digit',
  });
}

export function calculateAge(dateOfBirth: string | Date): number {
  const dob = typeof dateOfBirth === 'string' ? new Date(dateOfBirth) : dateOfBirth;
  const today = new Date();

  let age = today.getFullYear() - dob.getFullYear();
  const monthDiff = today.getMonth() - dob.getMonth();

  if (monthDiff < 0 || (monthDiff === 0 && today.getDate() < dob.getDate())) {
    age--;
  }

  return age;
}

export function formatAge(dateOfBirth: string | Date): string {
  const age = calculateAge(dateOfBirth);

  if (age === 0) {
    const dob = typeof dateOfBirth === 'string' ? new Date(dateOfBirth) : dateOfBirth;
    const today = new Date();
    const months =
      (today.getFullYear() - dob.getFullYear()) * 12 + today.getMonth() - dob.getMonth();

    if (months === 0) {
      const days = Math.floor((today.getTime() - dob.getTime()) / (1000 * 60 * 60 * 24));
      return `${days} day${days !== 1 ? 's' : ''} old`;
    }

    return `${months} month${months !== 1 ? 's' : ''} old`;
  }

  return `${age} year${age !== 1 ? 's' : ''} old`;
}
