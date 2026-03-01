use super::User;

/// A trait signifying items in the wiki which users are able to post and sometimes modify, such as art or characters.
pub trait UsermadePost {
    /// Returns whether the given item can be modified by a certain user.
    /// Default implementation returns true for users that can modify items by other users.
    fn can_be_modified_by(&self, user: &User) -> bool {
        user.user_type.permissions().can_modify_others_content
    }

    /// Returns whether the given item can be modified by a certain user. If None, returns false.
    /// A wrapper around can_be_modified_by because I need to unwrap option very often. Don't reimplement this!
    fn can_optionally_be_modified_by(&self, user: &Option<User>) -> bool {
        user.as_ref()
            .is_some_and(|user| self.can_be_modified_by(user))
    }
}
