use entity::Entity;
use cgmath::{Vector3, Point3};
use physics::Contact;

pub struct ContactCollection {
    contacts: Vec<Contact>
}

impl ContactCollection {
    pub fn new() -> Self {
        ContactCollection { contacts: Vec::new() }
    }

    pub fn clear_contacts(&mut self) {
        self.contacts.clear();
    }

    pub fn push_contact(&mut self, contact: Contact) {
        self.contacts.push(contact);
    }

    pub fn contacts<'a>(&'a self) -> &'a [Contact] {
        self.contacts.as_slice()
    }
}
